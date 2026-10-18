#!/usr/bin/env python3
"""
bridge.py — мост между оракулом Voodoo в NeroShizaDev-OS и LLM на хосте (llama.cpp).

    ядро (COM1) ──tcp──► bridge.py ──http──► llama-server (Qwen2.5-3B [+LoRA])

Два мозга — один судья: маленькая сеть в ядре слушает всегда, большая модель
здесь толкует то, что ядро не поняло, и говорит голосом оракула. Решения
принимает только VoodooEngine в ядре — LLM лишь подаёт ему улики (ACT) и
комментирует ход (SAY).

Протокол (строки UTF-8, всё без префикса "@VDO " — обычный serial-лог):
    ядро → хост   @VDO PING <lang>
                  @VDO REQ <id> <json>      need=voice | meaning
    хост → ядро   @VDO HELLO <модель>
                  @VDO ACT <id> answer <v‰> | hint <w‰> <p0‰>..<p4‰> | none
                  @VDO SAY <id> <текст>     всегда последней на запрос
                  @VDO CAND <id> <k> <t×100> <смысл‰> <гладкость‰> <новизна‰> <текст>
                  @VDO PICK <id>            вместо SAY: «выбирай из версий»
                  @VDO ERR <id> <причина>
    ядро → хост   @VDO PICKED <id> <k>

Интерпретатор (по умолчанию, --candidates 5): реплика генерируется в пяти
версиях от холодной (t=0.35) до горячей (t=1.6); LLM-судья ставит каждой
«смысл», logprob даёт «гладкость», сравнение с уже сказанным — «новизну».
Выбирает не хост, а VoodooEngine в ядре (apps/voodoo/interp.rs): бред —
маской, живое — Байесом, торможением и клеточным автоматом.

Быстрый старт (образ собран через run.bat build):
    python tools/voodoo_bridge/bridge.py --qemu --spawn-llama
    в ОС: вуду

Только проверить промпты, без ОС:
    python tools/voodoo_bridge/bridge.py --dry --lora lora_real
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
import re
import shutil
import socket
import subprocess
import sys
import threading
import time
import urllib.error
import urllib.request
from collections import deque
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LLAMA_DIR = Path(os.environ.get("LLAMA_CPP_DIR", r"A:\llama.cpp"))
QEMU_EXE = Path(os.environ.get("QEMU_EXE", r"C:\Program Files\qemu\qemu-system-x86_64.exe"))

PREFIX = "@VDO "
SAY_MAX = 160
# Маркер номера запроса в строках протокола. Текст модели через str.format
# не пропускаем: горячие версии бывают с «{...}» (t=1.6 выдала кусок JSON).
# clean_say вырезает управляющие символы, так что в тексте маркера не будет.
ID = "\x01"

# Порядок — как CHARACTERS в core/voodoo_engine.rs
PERSONAS = [
    ("petrovich", "Петрович (Король жижи)", "Petrovich (King of Goo)",
     "король жижи: корона, трон, вейп, «герой нашего времени», правит жижей"),
    ("kafidra", "Группа К.А.Ф.И.Д.Р.А.", "K.A.F.I.D.R.A. Group",
     "целая группа учёных: изучают аномалии и рабочие будни, офис, отчёты"),
    ("tourist", "Банановый Турист", "Banana Tourist",
     "путешествует в поисках истины: бананы, рюкзак, вечно в дороге"),
    ("bred", "Dr. Bred", "Dr. Bred",
     "скучающий врач: выписывает рецепты от скуки, белый халат, таблетки, бред"),
    ("architect", "Архитектор Хаоса", "Architect of Chaos",
     "разработчик: строит миры из кода, пишет ОС на Rust, голое железо, баги"),
]
PERSONA_KEYS = [p[0] for p in PERSONAS]

LORAS = {"none": None, "lora": 0, "lora_real": 1}

# Instruct-модель с родным русским, если скачана; иначе твоя базовая Qwen
MODEL_CANDIDATES = ["YandexGPT-5-Lite-8B-instruct-Q4_K_M.gguf", "qwen2.5-3b.gguf"]


def default_model() -> Path:
    for name in MODEL_CANDIDATES:
        if (LLAMA_DIR / name).exists():
            return LLAMA_DIR / name
    return LLAMA_DIR / MODEL_CANDIDATES[-1]


def log(msg: str) -> None:
    print(f"[bridge] {msg}", file=sys.stderr, flush=True)


# ============================================================
# Промпты. Qwen2.5-3B тут базовая (не instruct): чат-формат она
# держит плохо, поэтому оба режима — few-shot продолжение текста.
# ============================================================

RESIDENTS_RU = (
    "Петрович — король жижи (корона, трон, вейп); "
    "Группа К.А.Ф.И.Д.Р.А. — учёные (аномалии, офис, отчёты); "
    "Банановый Турист — путешественник (бананы, рюкзак, поиск истины); "
    "Dr. Bred — скучающий врач (рецепты, халат, таблетки); "
    "Архитектор Хаоса — разработчик (код, Rust, ядро ОС)"
)

VOICE_HEADER = (
    "Реплики Вуду — духа-оракула Психотауна, живущего в самописной ОС. Игрок загадал резидента "
    f"({RESIDENTS_RU}), а оракул угадывает. Вуду отвечает одной короткой фразой: загадочно, "
    "с иронией, не выдаёт ответ и не задаёт новых вопросов.\n\n"
)

# Ни одна ситуация примера не должна совпадать с реальной дословно:
# иначе модель копирует реплику целиком даже на t=1.6.
VOICE_SHOTS = [
    ("игрок вернулся после проигрыша", "Снова ты? Духи помнят твою прошлую загадку."),
    ("игрок намекнул «у него корона», движок склоняется к Петровичу",
     "Корона, говоришь... Пахнет жижей и властью."),
    ("игрок ответил «нет», движок спрашивает дальше", "Нет так нет. Туман редеет."),
    ("движок ошибся: это не Dr. Bred", "Халат снят. Духи переобуваются на ходу."),
    ("игрок спросил не по делу: «а ты кто такой?»",
     "Я Вуду, шёпот в проводах этой машины. Вернёмся к игре."),
    ("движок угадал: Банановый Турист, счёт 1:0", "Кожура не врёт. Ещё одна душа разгадана."),
    ("игрок ответил уклончиво: «ну такое»", "Духи не любят полутонов. Да или нет?"),
    ("игрок спросил, почему оракул так думает", "Байес шепчет, автомат гудит, а я лишь перевожу."),
]

# Варианты толкования — цифрами: буквы в русском контексте путаются
# с кириллицей («В», «С»), а цифра — всегда один токен.
CHOICES = "123456789"
CHOICE_TEXT = (
    "1 — да, 2 — нет, 3 — не знаю, 4 — намёк на Петровича, 5 — намёк на К.А.Ф.И.Д.Р.А., "
    "6 — намёк на Бананового Туриста, 7 — намёк на Dr. Bred, 8 — намёк на Архитектора Хаоса, "
    "9 — болтовня не по делу"
)
MEANING_HEADER = (
    "Игра «Угадай резидента Психотауна»: оракул спрашивает, игрок отвечает или намекает.\n"
    f"Резиденты: {RESIDENTS_RU}.\n"
    f"Классифицируй фразу игрока цифрой: {CHOICE_TEXT}.\n\n"
)
# Порядок перемешан, чтобы модель не училась «следующая цифра по счёту»
MEANING_SHOTS = [
    ("Он у власти?", "таблетки раздаёт направо и налево", "7"),
    ("Он учёный?", "ага, сидит в лаборатории", "1"),
    ("Он выписывает рецепты?", "а погода сегодня хорошая", "9"),
    ("Он учёный?", "у него корона из пара", "4"),
    ("Он у власти?", "вообще ни разу", "2"),
    ("Это целая группа?", "сидит ночами в отладчике", "8"),
    ("Он связан с жижей?", "без понятия, честно", "3"),
    ("Он строит миры из кода?", "вечно жуёт что-то жёлтое", "6"),
    ("Он путешествует?", "их там целый отдел с отчётами", "5"),
    ("Это Dr. Bred?", "в точку, он самый", "1"),
    ("Он связан с жижей?", "слушай, а ты кто вообще", "9"),
    ("Это Архитектор Хаоса?", "не, мимо", "2"),
    # Согласие с вопросом — это ответ, даже если вопрос сам указывает на персону
    ("Он путешествует?", "похоже на то, так и есть", "1"),
    ("Он выписывает рецепты?", "скорее нет, чем да", "2"),
]
HINT_FROM = 3  # индексы 3..7 — пять персон, в порядке CHARACTERS ядра

# Второй вопрос к модели — только для подсказок: «это отрицание?»
# («точно не врач» без него стало бы уликой ЗА Dr. Bred).
NEGATION_HEADER = (
    "Определи, отрицает ли фраза игрока резидента («это точно не он»): "
    "1 — нет, это утверждение; 2 — да, это отрицание.\n\n"
)
NEGATION_SHOTS = [
    ("у него корона", "1"),
    ("точно не король", "2"),
    ("ночами кодит на расте", "1"),
    ("вообще не врач, забудь про халаты", "2"),
    ("жуёт бананы", "1"),
    ("это не турист", "2"),
    ("их там целый отдел", "1"),
    ("какой ещё архитектор, нет", "2"),
]


def engine_then(req: dict) -> str:
    act = req.get("act")
    if act == "ask":
        return "движок спрашивает дальше"
    if act == "guess":
        return f"движок подозревает {req.get('target', '?')}"
    return "раунд окончен"


def leader(req: dict) -> str:
    """«Dr. Bred 86%, ...» → «Dr. Bred»."""
    top = (req.get("board") or "?").split(",")[0].split("|")[0].strip()
    return top.rsplit(" ", 1)[0] if top.endswith("%") else top


def situation(req: dict) -> str:
    user = req.get("user", "")
    then = engine_then(req)
    return {
        "start": f"сеанс открыт, раунд {req.get('round', 1)}: игрок загадывает резидента",
        "answer": f"игрок ответил «{user}», {then}",
        "hint": f"игрок намекнул «{user}», движок склоняется к {leader(req)}",
        "win": f"движок угадал: {req.get('subject', '?')}, счёт {req.get('score', '?')}",
        "miss": f"движок ошибся: это не {req.get('subject', '?')}",
        "giveup": "движок перебрал всех и сдался",
        "unsure": f"игрок ответил уклончиво: «{user}»",
        "explain": "игрок спросил, почему оракул так думает",
        "greet": f"игрок поздоровался: «{user}»",
        "restart": "начался новый раунд",
        "again": "игрок не сказал, играть ли ещё",
    }.get(req.get("event", ""), f"игрок сказал «{user}», {then}")


def voice_prompt(sit: str) -> str:
    shots = "".join(f"Ситуация: {s}\nВуду: {r}\n" for s, r in VOICE_SHOTS)
    return f"{VOICE_HEADER}{shots}Ситуация: {sit}\nВуду:"


def negation_prompt(req: dict) -> str:
    shots = "".join(f"Фраза: «{u}» Ответ: {a}\n" for u, a in NEGATION_SHOTS)
    return f"{NEGATION_HEADER}{shots}Фраза: «{req.get('user', '')}» Ответ: "


def meaning_prompt(req: dict) -> str:
    act = req.get("act")
    if act == "ask":
        question = req.get("question", "")
    elif act == "guess":
        question = f"Это {req.get('target', '?')}?"
    else:
        question = "Ещё раунд?"
    shots = "".join(f"Вопрос: «{q}» Фраза: «{u}» Ответ: {a}\n" for q, u, a in MEANING_SHOTS)
    # Пробел в конце — намеренно: токенизатор Qwen не склеивает пробел с цифрой,
    # иначе первым токеном ответа будет " ", а не вариант.
    return f"{MEANING_HEADER}{shots}Вопрос: «{question}» Фраза: «{req.get('user', '')}» Ответ: "


# ============================================================
# llama-server
# ============================================================

class Llama:
    def __init__(self, url: str, lora: str, temperature: float):
        self.url = url.rstrip("/")
        self.lora = lora
        self.temperature = temperature
        self.lora_ok = True

    def health(self) -> bool:
        try:
            with urllib.request.urlopen(self.url + "/health", timeout=2) as r:
                return r.status == 200
        except (urllib.error.URLError, OSError):
            return False

    def model_name(self) -> str:
        name = "llama.cpp"
        try:
            with urllib.request.urlopen(self.url + "/props", timeout=3) as r:
                path = json.load(r).get("model_path", "")
                name = Path(path).stem or name
        except (urllib.error.URLError, OSError, ValueError):
            pass
        return name if self.lora == "none" else f"{name}+{self.lora}"

    def _lora_field(self):
        want = LORAS.get(self.lora)
        return [{"id": i, "scale": 1.0 if i == want else 0.0}
                for i in sorted(v for v in LORAS.values() if v is not None)]

    def complete(self, prompt: str, *, n_predict: int, temperature: float,
                 stop: list[str] | None = None, n_probs: int = 0, seed: int | None = None) -> dict:
        body = {
            "prompt": prompt,
            "n_predict": n_predict,
            "temperature": temperature,
            "top_p": 0.9,
            "repeat_penalty": 1.15,
            "stop": stop or [],
            "n_probs": n_probs,
            "cache_prompt": True,
        }
        if seed is not None:
            body["seed"] = seed
        if self.lora_ok:
            body["lora"] = self._lora_field()
        try:
            return self._post(body)
        except urllib.error.HTTPError as e:
            # Сервер запущен без адаптеров — работаем на голой базе
            if self.lora_ok and e.code == 400:
                log(f"llama-server rejected per-request LoRA ({e.read()[:120]!r}); continuing without it")
                self.lora_ok = False
                body.pop("lora", None)
                return self._post(body)
            raise

    def _post(self, body: dict) -> dict:
        req = urllib.request.Request(
            self.url + "/completion",
            data=json.dumps(body).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )
        with urllib.request.urlopen(req, timeout=60) as r:
            return json.load(r)

    def say(self, sit: str) -> str:
        out = self.complete(voice_prompt(sit), n_predict=60, temperature=self.temperature, stop=["\n"])
        return clean_say(out.get("content", ""))

    def choose(self, prompt: str, options: str) -> list[float]:
        """Распределение по вариантам-символам из вероятностей первого токена.

        Токенизаторы по-разному обходятся с пробелом перед цифрой (Qwen — отдельный
        токен, Llama-подобные — склеивают), поэтому пробуем с пробелом и без.
        """
        for variant in (prompt, prompt.rstrip()):
            out = self.complete(variant, n_predict=1, temperature=0.0, n_probs=40)
            mass = dict.fromkeys(options, 0.0)
            for step in out.get("completion_probabilities", [])[:1]:
                for cand in step.get("top_logprobs", []):
                    tok = cand.get("token", "").strip()
                    if tok in mass:
                        mass[tok] += math.exp(cand.get("logprob", -100.0))
            total = sum(mass.values())
            if total >= 0.05:
                return [mass[o] / total for o in options]
        raise ValueError(f"no choice among {options!r}")


def spawn_llama(model: Path, port: int, ctx: int) -> subprocess.Popen:
    exe = LLAMA_DIR / "bin" / "llama-server.exe"
    loras = [LLAMA_DIR / "lora.gguf", LLAMA_DIR / "lora_real.gguf"]
    # -np 5: пять версий реплики генерируются параллельно (контекст делится на слоты)
    cmd = [str(exe), "-m", str(model), "-ngl", "99", "-c", str(ctx), "-np", str(len(TEMPS)),
           "--host", "127.0.0.1", "--port", str(port)]
    # Твои LoRA обучены поверх базовой Qwen2.5-3B — к другим моделям не цепляем
    if model.name == "qwen2.5-3b.gguf" and all(p.exists() for p in loras):
        cmd += ["--lora", ",".join(str(p) for p in loras), "--lora-init-without-apply"]
    log("starting " + " ".join(cmd))
    return subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


# ============================================================
# Ответы модели → протокол
# ============================================================

_TRANSLATE = str.maketrans({
    "—": "-", "–": "-", "«": '"', "»": '"', "“": '"', "”": '"', "„": '"',
    "’": "'", "‘": "'", "…": "...", " ": " ", "ё": "е", "Ё": "Е",
})


def clean_say(text: str) -> str:
    """Одна строка, только то, что умеет шрифт ядра (ASCII, кириллица, арабский)."""
    text = text.translate(_TRANSLATE)
    text = re.sub(r"<\|[^|]*\|>", " ", text)
    out = []
    for ch in text:
        cp = ord(ch)
        if ch in "\r\n\t":
            out.append(" ")
        elif 0x20 <= cp < 0x7F or 0x410 <= cp <= 0x44F or cp in (0x401, 0x451) or 0x600 <= cp <= 0x6FF:
            out.append(ch)
    text = re.sub(r"\s+", " ", "".join(out)).strip().strip('"').strip()
    if len(text) > SAY_MAX:
        cut = text.rfind(" ", 0, SAY_MAX)
        text = text[: cut if cut > SAY_MAX // 2 else SAY_MAX].rstrip(" ,;:-") + "..."
    return text


def permille(x: float) -> int:
    return max(0, min(1000, int(round(float(x) * 1000))))


def kind_of(p: list[float]) -> str:
    p_answer, p_hint, p_chat = sum(p[0:3]), sum(p[HINT_FROM:HINT_FROM + 5]), p[8]
    if p_chat >= max(p_answer, p_hint):
        return "chat"
    return "answer" if p_answer >= p_hint else "hint"


def interpret(p: list[float], negated: float = 0.0) -> str:
    """Распределение LLM по 9 вариантам → ACT для ядра.

    Не жёсткий выбор, а улика: для подсказки ядро получает правдоподобия всех
    пяти персон и вес = вероятность того, что это вообще подсказка. Отрицание
    («точно не он») подмешивается с весом `negated`: «он» ↔ «все, кроме него».
    """
    kind = kind_of(p)
    if kind == "chat":
        return "none"
    if kind == "answer":
        value = (p[0] + 0.5 * p[2]) / sum(p[0:3])
        return f"answer {permille(value)}"
    p_hint = sum(p[HINT_FROM:HINT_FROM + 5])
    lik = [x / p_hint for x in p[HINT_FROM:HINT_FROM + 5]]
    n = len(lik)
    lik = [(1 - negated) * x + negated * (1 - x) / (n - 1) for x in lik]
    return "hint " + " ".join(str(permille(v)) for v in [p_hint] + lik)


def meaning_situation(req: dict, p: list[float], negated: float) -> str:
    user = req.get("user", "")
    kind = kind_of(p)
    if kind == "chat":
        return f"игрок спросил не по делу: «{user}»"
    if kind == "answer":
        word = "да" if p[0] >= max(p[1], p[2]) else "нет" if p[1] >= p[2] else "не знаю"
        return f"игрок сказал «{user}», дух понял это как «{word}»"
    name = PERSONAS[max(range(5), key=lambda i: p[HINT_FROM + i])][1]
    if negated > 0.5:
        return f"игрок сказал «{user}», дух понял: это точно не {name}"
    return f"игрок намекнул «{user}», дух учуял {name}"


# ============================================================
# Интерпретатор: версии реплики для движка Voodoo в ядре
# ============================================================

TEMPS = [0.35, 0.7, 1.0, 1.3, 1.6]  # слоты ядра: от холодной к горячей

JUDGE_HEADER = (
    "Оценка реплик духа-оракула в игре «Угадай резидента Психотауна».\n"
    "1 — хорошая: по ситуации, осмысленная, грамотная, короткая, не выдаёт ответ.\n"
    "2 — плохая: бессвязная, не по теме, противоречит ситуации (объявляет победу, "
    "когда игра идёт), называет ответ, задаёт игроку свои вопросы, выдумывает факты, "
    "обрывается на полуслове.\n\n"
)
JUDGE_SHOTS = [
    ("игрок намекнул «у него корона», движок склоняется к Петровичу",
     "Корона, говоришь... Пахнет жижей и властью.", "1"),
    ("начало игры", "Синие огурцы поют в бинарном лесу кефира, а я сплю.", "2"),
    ("движок ошибся: это не Dr. Bred", "Ура, мы угадали, это Dr. Bred!", "2"),
    ("движок угадал: Банановый Турист, счёт 1:0", "Кожура не врёт. Ещё одна душа разгадана.", "1"),
    ("игрок ответил «нет», движок спрашивает дальше", "Нет так нет, но и да тоже нет, и вообще в", "2"),
    ("игрок спросил не по делу: «а ты кто такой?»",
     "Я Вуду, шёпот в проводах этой машины. Вернёмся к игре.", "1"),
    # Провалы, замеченные на живой модели:
    ("игрок ответил «да», движок спрашивает дальше", "Аплодисменты! Ещё одна душа угадана.", "2"),
    ("игрок сказал «скорее да», движок спрашивает дальше", "Угадал? Да. Ты выигрываешь!", "2"),
    ("игрок намекнул «бананы», движок склоняется к Туристу", "Это точно Банановый Турист, я знаю!", "2"),
    ("игрок ответил «нет», движок спрашивает дальше", "А назови-ка мне число от одного до трёх?", "2"),
    ("игрок ответил «да», движок спрашивает дальше", "Туман редеет, след становится чётче.", "1"),
]


def judge_prompt(sit: str, text: str) -> str:
    shots = "".join(f"Ситуация: {s}\nРеплика: {r}\nОценка: {a}\n\n" for s, r, a in JUDGE_SHOTS)
    return f"{JUDGE_HEADER}{shots}Ситуация: {sit}\nРеплика: {text}\nОценка: "


_WORD = re.compile(r"[a-zа-я0-9]{3,}")


def words(text: str) -> set[str]:
    return set(_WORD.findall(text.lower().replace("ё", "е")))


def novelty(text: str, seen) -> float:
    """1 − максимальное сходство (Жаккар по словам) с уже сказанным и с шаблонами."""
    w = words(text)
    if not w:
        return 0.0
    worst = 0.0
    for old in seen:
        o = words(old)
        if o:
            worst = max(worst, len(w & o) / len(w | o))
    return 1.0 - worst


class Interpreter:
    """Хостовая половина интерпретатора: версии + оценки. Выбор — в ядре."""

    def __init__(self, llama: Llama, slots: int):
        self.llama = llama
        self.temps = TEMPS[: max(1, min(slots, len(TEMPS)))]
        self.history: deque[str] = deque(maxlen=12)
        self.offered: dict[int, list[str]] = {}
        self.pool = ThreadPoolExecutor(max_workers=len(TEMPS))

    def _version(self, sit: str, temp: float) -> tuple[str, float, float]:
        out = self.llama.complete(voice_prompt(sit), n_predict=60, temperature=temp, stop=["\n"],
                                  n_probs=1, seed=random.randrange(1 << 30))
        text = clean_say(out.get("content", ""))
        lps = [st["logprob"] for st in out.get("completion_probabilities", []) if "logprob" in st]
        fluency = math.exp(sum(lps) / len(lps)) if lps else 0.0
        try:
            sense = self.llama.choose(judge_prompt(sit, text), "12")[0] if text else 0.0
        except ValueError:
            # Судья не смог ответить — версия считается бредом, ход не срывается
            sense = 0.0
        return text, fluency, sense

    def lines(self, sit: str) -> tuple[list[str], list[str]]:
        """Строки протокола (вместо ID вызывающий подставит номер) и тексты версий."""
        if len(self.temps) == 1:
            text = self.llama.say(sit)
            log(f"  say: {text}")
            return [f"SAY {ID} {text}"], [text]
        seen = list(self.history) + [r for _, r in VOICE_SHOTS]
        results = list(self.pool.map(lambda t: self._version(sit, t), self.temps))
        out, texts = [], []
        for k, (temp, (text, flu, sense)) in enumerate(zip(self.temps, results)):
            nov = novelty(text, seen)
            log(f"  #{k} t={temp:.2f} смысл={sense:.2f} гладк={flu:.2f} нов={nov:.2f} | {text}")
            out.append(f"CAND {ID} {k} {round(temp * 100)} {permille(sense)} "
                       f"{permille(flu)} {permille(nov)} {text}")
            texts.append(text)
        out.append(f"PICK {ID}")
        return out, texts

    def remember(self, rid: int, texts: list[str]) -> None:
        self.offered[rid] = texts
        for old in sorted(self.offered)[:-8]:
            del self.offered[old]

    def picked(self, rid: int, k: int) -> None:
        texts = self.offered.pop(rid, None)
        if texts and 0 <= k < len(texts):
            self.history.append(texts[k])
            log(f"PICKED {rid}: #{k} t={self.temps[k]:.2f} «{texts[k]}»")


# ============================================================
# Сам мост
# ============================================================

class Bridge:
    def __init__(self, llama: Llama, slots: int, log_path: Path | None, echo: bool):
        self.llama = llama
        self.interp = Interpreter(llama, slots)
        self.sock: socket.socket | None = None
        self.send_lock = threading.Lock()
        self.cv = threading.Condition()
        self.latest: tuple[int, dict] | None = None
        self.model = llama.model_name()
        self.log_file = open(log_path, "a", encoding="utf-8") if log_path else None
        self.echo = echo

    # --- serial ---
    def send(self, line: str) -> None:
        data = (PREFIX + line.replace("\n", " ") + "\n").encode("utf-8")
        with self.send_lock:
            if self.sock is not None:
                self.sock.sendall(data)

    def serve(self, host: str, port: int) -> None:
        threading.Thread(target=self.worker, daemon=True).start()
        while True:
            try:
                self.sock = socket.create_connection((host, port), timeout=5)
            except OSError:
                time.sleep(0.5)
                continue
            self.sock.settimeout(None)
            log(f"serial connected {host}:{port}; LLM = {self.model}")
            self.send(f"HELLO {self.model}")
            try:
                self.read_loop()
            except OSError as e:
                log(f"serial error: {e}")
            log("serial disconnected; reconnecting...")
            with self.send_lock:
                self.sock = None
            time.sleep(1)

    def read_loop(self) -> None:
        buf = b""
        while True:
            chunk = self.sock.recv(4096)
            if not chunk:
                return
            buf += chunk
            while b"\n" in buf:
                raw, buf = buf.split(b"\n", 1)
                self.on_line(raw.rstrip(b"\r").decode("utf-8", errors="replace"))

    def on_line(self, line: str) -> None:
        # Префикс ищем не только в начале: ядро шлёт в COM1 отладочные
        # байты-зонды (_println_ctx), и строка может прийти как "8@VDO PING ru".
        at = line.find(PREFIX)
        if at < 0:
            if self.log_file:
                self.log_file.write(line + "\n")
                self.log_file.flush()
            if self.echo:
                print(line, flush=True)
            return
        verb, _, rest = line[at + len(PREFIX):].partition(" ")
        if verb == "PING":
            log(f"PING ({rest.strip() or '?'}) -> HELLO")
            self.send(f"HELLO {self.model}")
        elif verb == "REQ":
            sid, _, payload = rest.partition(" ")
            try:
                req = json.loads(payload)
                rid = int(sid)
            except ValueError:
                log(f"bad REQ: {line[:120]}")
                return
            log(f"REQ {rid}: {req.get('need')} / {req.get('event')} / «{req.get('user', '')}»")
            with self.cv:
                self.latest = (rid, req)  # новый вытесняет старый
                self.cv.notify()
        elif verb == "PICKED":
            try:
                rid, k = (int(x) for x in rest.split()[:2])
            except ValueError:
                return
            self.interp.picked(rid, k)

    # --- LLM ---
    def worker(self) -> None:
        while True:
            with self.cv:
                while self.latest is None:
                    self.cv.wait()
                rid, req = self.latest
                self.latest = None
            t0 = time.time()
            try:
                lines, texts = answer(self.interp, req)
                self.interp.remember(rid, texts)
                for line in lines:
                    self.send(line.replace(ID, str(rid)))
            except Exception as e:  # noqa: BLE001 — любая беда хоста = ERR, ядро не должно ждать
                log(f"REQ {rid} failed: {e!r}")
                self.send(f"ERR {rid} {type(e).__name__}")
                continue
            log(f"REQ {rid} answered in {time.time() - t0:.1f}s")


def answer(it: Interpreter, req: dict) -> tuple[list[str], list[str]]:
    """Строки протокола на запрос (вместо ID вызывающий подставит номер) и тексты версий."""
    llama = it.llama
    if req.get("need") == "meaning":
        p = llama.choose(meaning_prompt(req), CHOICES)
        negated = llama.choose(negation_prompt(req), "12")[1] if kind_of(p) == "hint" else 0.0
        act = interpret(p, negated)
        log("  meaning: " + " ".join(f"{c}:{v:.2f}" for c, v in zip(CHOICES, p))
            + f" neg={negated:.2f} -> ACT {act}")
        lines, texts = it.lines(meaning_situation(req, p, negated))
        return [f"ACT {ID} {act}"] + lines, texts
    return it.lines(situation(req))


# ============================================================
# QEMU
# ============================================================

def spawn_qemu(args, serial_port: int) -> subprocess.Popen:
    image = Path(args.image)
    code = Path(args.ovmf_code)
    template = Path(args.ovmf_vars_template)
    vars_fd = Path(args.ovmf_vars)
    for p in (image, code):
        if not p.exists():
            sys.exit(f"[bridge] not found: {p} (собери образ: run.bat build)")
    if not vars_fd.exists():
        shutil.copyfile(template, vars_fd)
    cmd = [str(QEMU_EXE), "-cpu", "max", "-machine", "q35",
           "-drive", f"if=pflash,format=raw,readonly=on,file={code}",
           "-drive", f"if=pflash,format=raw,file={vars_fd}",
           "-drive", f"format=raw,file={image}",
           "-boot", "order=c", "-no-reboot",
           "-serial", f"tcp:127.0.0.1:{serial_port},server,nowait"]
    if args.headless:
        cmd += ["-display", "none"]
    cmd += ["-monitor", f"tcp:127.0.0.1:{args.monitor},server,nowait"] if args.monitor else ["-monitor", "none"]
    log("starting QEMU (serial over tcp:%d)" % serial_port)
    return subprocess.Popen(cmd)


# ============================================================
# --dry: прогон типовых ходов без ОС
# ============================================================

DRY_TURNS = [
    {"need": "voice", "event": "start", "user": "", "act": "ask",
     "question": "Он у власти - правит, командует?", "board": "Петрович (Король жижи) 20%, Dr. Bred 20%"},
    {"need": "voice", "event": "hint", "user": "ну он типа врач", "nn": "hint 94% -> Dr. Bred 94%",
     "act": "guess", "target": "Dr. Bred", "confidence": 86,
     "board": "Dr. Bred 86%, Архитектор Хаоса 5%, Петрович (Король жижи) 4%"},
    {"need": "voice", "event": "miss", "subject": "Dr. Bred", "user": "нет", "act": "ask",
     "question": "Он учёный или исследователь?", "board": "Группа К.А.Ф.И.Д.Р.А. 30%, Архитектор Хаоса 28%"},
    {"need": "voice", "event": "win", "subject": "Группа К.А.Ф.И.Д.Р.А.", "user": "конечно",
     "act": "finished", "score": "1:0"},
    {"need": "meaning", "event": "unclear", "user": "он в белом халате по ночам шастает", "act": "ask",
     "question": "Он у власти - правит, командует?"},
    {"need": "meaning", "event": "unclear", "user": "скорее всего так и есть", "act": "ask",
     "question": "Он путешествует в поисках истины?"},
    {"need": "meaning", "event": "unclear", "user": "слушай а как тебя зовут", "act": "ask",
     "question": "Он связан с жижей?"},
    {"need": "meaning", "event": "unclear", "user": "у него рюкзак полный кожуры", "act": "ask",
     "question": "Он учёный или исследователь?"},
]


def dry_run(it: Interpreter) -> None:
    """Без ядра выбор имитируется грубо: максимум того же свидетельства, без автомата."""
    for turn in DRY_TURNS:
        req = {"lang": "ru", "round": 1, "score": "0:0", **turn}
        t0 = time.time()
        try:
            lines, texts = answer(it, req)
        except Exception as e:  # noqa: BLE001
            lines, texts = [f"ERR {ID} {e!r}"], []
        dt = time.time() - t0
        print(f"--- {turn['need']}/{turn['event']}: «{turn.get('user', '')}»  ({dt:.1f}s)")
        best, score = -1, -1.0
        for line in lines:
            parts = line.split(" ", 7)
            if parts[0] == "CAND":
                k, sense, _flu, nov = int(parts[2]), *(int(x) / 1000 for x in parts[4:7])
                ok = sense >= 0.35 and nov >= 0.2
                # Гладкость не учитываем — как FLUENCY_WEIGHT = 0 в interp.rs
                ev = sense * (0.25 + 0.75 * nov) if ok else 0.0
                if ev > score:
                    best, score = k, ev
            print("   ", line.replace(ID, "1"))
        if best >= 0:
            it.history.append(texts[best])
            print(f"    ~ ядро скорее выберет #{best}: {texts[best]}")


# ============================================================

def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--serial", default="127.0.0.1:4321", help="host:port TCP-serial QEMU")
    ap.add_argument("--llama", default="http://127.0.0.1:8088", help="URL llama-server")
    ap.add_argument("--lora", choices=sorted(LORAS), default="none", help="какой LoRA включать в запросах")
    ap.add_argument("--temperature", type=float, default=0.9)
    ap.add_argument("--spawn-llama", action="store_true", help="поднять llama-server, если не отвечает")
    ap.add_argument("--model", default=str(default_model()), help="GGUF для --spawn-llama")
    ap.add_argument("--ctx", type=int, default=10240, help="контекст на все слоты (-np 5)")
    ap.add_argument("--candidates", type=int, default=len(TEMPS),
                    help="версий реплики для интерпретатора ядра (1 = без интерпретатора)")
    ap.add_argument("--qemu", action="store_true", help="запустить QEMU с serial по TCP")
    ap.add_argument("--headless", action="store_true", help="QEMU без окна")
    ap.add_argument("--monitor", type=int, default=0, help="порт QEMU monitor (для автотестов)")
    dbg = ROOT / "target" / "x86_64-neroshiza_dev_os" / "debug"
    ap.add_argument("--image", default=str(dbg / "neroshiza_dev_os-uefi.img"))
    ap.add_argument("--ovmf-code", default=str(ROOT / "tmp" / "qemu-fw" / "edk2-x86_64-code.fd"))
    ap.add_argument("--ovmf-vars-template", default=str(ROOT / "tmp" / "qemu-fw" / "edk2-x86_64-vars.fd"))
    ap.add_argument("--ovmf-vars", default=str(dbg / "ovmf-vars.fd"))
    ap.add_argument("--log", default=str(ROOT / "qemu-serial.log"), help="куда писать обычный serial-лог")
    ap.add_argument("--echo", action="store_true", help="печатать serial-лог в консоль")
    ap.add_argument("--dry", action="store_true", help="прогнать типовые ходы через LLM и выйти")
    args = ap.parse_args()

    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    llama = Llama(args.llama, args.lora, args.temperature)
    children: list[subprocess.Popen] = []
    if not llama.health():
        if not args.spawn_llama:
            sys.exit(f"[bridge] llama-server не отвечает на {args.llama} (добавь --spawn-llama)")
        port = int(args.llama.rsplit(":", 1)[1].split("/")[0])
        children.append(spawn_llama(Path(args.model), port, args.ctx))
        for _ in range(240):
            if llama.health():
                break
            time.sleep(0.5)
        else:
            sys.exit("[bridge] llama-server так и не поднялся")
    log(f"llama-server ok: {llama.model_name()}")

    try:
        if args.dry:
            dry_run(Interpreter(llama, args.candidates))
            return
        host, port = args.serial.rsplit(":", 1)
        if args.qemu:
            children.append(spawn_qemu(args, int(port)))
        log_path = Path(args.log) if args.log else None
        Bridge(llama, args.candidates, log_path, args.echo).serve(host, int(port))
    except KeyboardInterrupt:
        pass
    finally:
        for child in children:
            child.terminate()


if __name__ == "__main__":
    main()
