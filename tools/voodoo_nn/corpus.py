"""
corpus.py — синтетический корпус для нейро-оракула Voodoo.

Две задачи на одном скрытом слое:
  * intent  — что человек сказал: да / нет / хз / подсказка / почему /
              заново / стоп / привет
  * persona — если это подсказка: про кого из резидентов Психотауна

Цели «мягкие»: «скорее да» = 70% yes + 30% maybe, «не доктор» =
равномерно по всем, кроме Dr. Bred. Так VoodooEngine получает не
жёсткий бит, а честную вероятность — и сам решает, что с ней делать.
"""

from __future__ import annotations

import random

from vnn import INTENTS, PERSONAS

I = {name: i for i, name in enumerate(INTENTS)}
P = {name: i for i, name in enumerate(PERSONAS)}

# ------------------------------------------------------------------
# Ответы на вопросы
# ------------------------------------------------------------------
YES = """
да|ага|угу|ну да|да да|конечно|естественно|точно|верно|правильно|именно
безусловно|разумеется|абсолютно|так точно|ещё бы|а то|йес|да, это так
ясное дело|само собой|однозначно да|сто процентов|сто пудов|сто пудово
бесспорно|в точку|да, конечно|ок|окей|ладно|давай|го|погнали|поехали
конечно да|точно да|да, верно|верно говоришь|в яблочко|угадал|он самый
yes|yeah|yep|yup|ya|sure|of course|definitely|absolutely|right|correct
true|indeed|affirmative|yes indeed|totally|for sure|exactly|that's right
ok|okay|you got it|bingo|correct guess|aye|yes it is
""".split("|")

NO = """
нет|неа|не|нет нет|нету|никак нет|ни в коем случае|конечно нет|нет конечно
ни за что|отнюдь|вовсе нет|совсем нет|ничего подобного|мимо|не угадал|неверно
неправильно|не так|точно нет|однозначно нет|не он|не она|не то|не это
нифига|ни разу|даже близко нет|нее|ноуп|нет, не так|не, не то|не-а|ошибся
no|nope|nah|no way|not at all|never|wrong|false|negative|not really|no no
definitely not|of course not|absolutely not|not him|not that|incorrect|nay
""".split("|")

MAYBE = """
не знаю|хз|без понятия|может быть|возможно|наверное|частично|иногда|отчасти
трудно сказать|сложно сказать|понятия не имею|затрудняюсь ответить|50 на 50
ну такое|фиг знает|кто его знает|не уверен|не уверена|как сказать|наполовину
и да и нет|вроде бы|бывает|может|пятьдесят на пятьдесят|не помню|чёрт знает
maybe|perhaps|possibly|i don't know|dunno|idk|not sure|kind of|sort of
partially|hard to say|sometimes|no idea|who knows|could be|50 50|i guess
""".split("|")

# Мягкие ответы: (текст, {intent: вес})
SOFT = [
    ("скорее да", {"yes": 0.7, "maybe": 0.3}),
    ("наверное да", {"yes": 0.7, "maybe": 0.3}),
    ("думаю да", {"yes": 0.75, "maybe": 0.25}),
    ("вроде да", {"yes": 0.7, "maybe": 0.3}),
    ("пожалуй да", {"yes": 0.75, "maybe": 0.25}),
    ("скорее всего да", {"yes": 0.75, "maybe": 0.25}),
    ("скорее нет", {"no": 0.7, "maybe": 0.3}),
    ("наверное нет", {"no": 0.7, "maybe": 0.3}),
    ("думаю нет", {"no": 0.75, "maybe": 0.25}),
    ("вроде нет", {"no": 0.7, "maybe": 0.3}),
    ("скорее всего нет", {"no": 0.75, "maybe": 0.25}),
    ("вряд ли", {"no": 0.65, "maybe": 0.35}),
    ("сомневаюсь", {"no": 0.55, "maybe": 0.45}),
    ("probably yes", {"yes": 0.7, "maybe": 0.3}),
    ("i think so", {"yes": 0.7, "maybe": 0.3}),
    ("mostly yes", {"yes": 0.7, "maybe": 0.3}),
    ("probably not", {"no": 0.7, "maybe": 0.3}),
    ("i don't think so", {"no": 0.7, "maybe": 0.3}),
    ("unlikely", {"no": 0.65, "maybe": 0.35}),
    ("doubt it", {"no": 0.6, "maybe": 0.4}),
]

# ------------------------------------------------------------------
# Управление диалогом
# ------------------------------------------------------------------
WHY = """
почему|почему так|объясни|поясни|как ты думаешь|что думаешь|покажи вероятности
расклад|статистика|что у тебя там|покажи расклад|покажи матрицу|кто лидирует
на кого думаешь|что видишь|твои мысли|отчёт|покажи|что там по вероятностям
why|explain|show stats|show probabilities|what do you think|status|stats
who is leading|show me|show matrix|your thoughts|report|what do you see
""".split("|")

RESTART = """
заново|сначала|ещё раз|давай ещё|новая игра|рестарт|сброс|начать сначала
давай заново|играем ещё|ещё партию|по новой|с начала|перезапуск|новый раунд
restart|again|new game|reset|one more|play again|start over|another round
""".split("|")

STOP = """
стоп|хватит|выход|выйти|пока|до свидания|закончить|всё, хватит|надоело|отмена
выключись|уйти|закрыть|свободен|конец|завершить|прощай|бывай|я ухожу|харэ
exit|quit|stop|bye|goodbye|enough|cancel|close|see you|leave|the end|end
""".split("|")

GREET = """
привет|здравствуй|здравствуйте|хай|ку|здорово|добрый день|добрый вечер|салют
ты кто|кто ты|как дела|как ты|хеллоу|приветствую|йо|доброе утро|что ты такое
hello|hi|hey|who are you|how are you|good morning|sup|yo|greetings|what are you
""".split("|")

# ------------------------------------------------------------------
# Подсказки про резидентов
#   phrases — готовые фразы-подсказки
#   nouns   — именные группы для шаблонов «это X», «не X»
# ------------------------------------------------------------------
HINTS = {
    "petrovich": {
        "phrases": """
король жижи|жижа|любит жижу|связан с жижей|король|царь|носит корону|корона
герой нашего времени|вейп|парит|пар|дым|трон|правит жижей|петрович|бутылка жижи
сидит на троне|правитель жижи|у него корона|жидкий король|главный по жиже
мужик в короне|властелин жижи|варит жижу|жижный царь|жижмейкер
king of goo|goo king|slime king|goo|slime|vape|crown|throne|hero of our time
petrovich|wears a crown|rules the goo|liquid king|the king
""",
        "nouns": "король|царь|король жижи|петрович|герой|вейпер|правитель|king|vaper",
    },
    "kafidra": {
        "phrases": """
группа|команда|коллектив|кафидра|кафедра|изучает аномалии|аномалии|рабочие будни
офис|отчёты|исследуют|учёные|лаборатория|анализ аномалий|их много|несколько человек
отдел|сотрудники|целая группа|изучают будни|научная группа|исследователи аномалий
пишут отчёты|сидят в офисе|аналитики|сборище учёных|их пятеро|группа к а ф и д р а
a group|team|group of people|anomalies|research team|office|reports|analysts
lab|they study anomalies|many people|scientists|department|crew|staff
""",
        "nouns": "группа|команда|коллектив|кафедра|отдел|учёные|аналитики|team|group",
    },
    "tourist": {
        "phrases": """
банан|бананы|турист|путешествует|ищет истину|истина|рюкзак|дорога|жёлтый
в пути|странник|ездит по миру|банановый|любит бананы|банановый турист|путник
ищет правду|путешественник|с рюкзаком|кожура|жёлтый турист|бродяга|паломник
гуляет по миру|в поисках истины|ест бананы|банановая кожура|скиталец
banana|bananas|tourist|travels|seeks the truth|truth|backpack|wanderer|yellow
traveller|on the road|likes bananas|banana tourist|journey|pilgrim|banana peel
""",
        "nouns": "турист|банан|странник|путешественник|бродяга|tourist|banana|traveller",
    },
    "bred": {
        "phrases": """
доктор|врач|рецепты|выписывает рецепты|таблетки|скука|от скуки|бред|медицина
больница|лечит|диагноз|доктор бред|доктор бредд|лекарства|медик|в халате|клиника
скучающий врач|пилюли|рецепт от скуки|ставит диагнозы|терапевт|психиатр|уколы
выписывает таблетки|белый халат|главврач|дохтур|лечит бредом
doctor|physician|prescriptions|pills|boredom|nonsense|medicine|hospital|dr bred
medic|writes prescriptions|diagnosis|clinic|white coat|therapist|drugs|bred
""",
        "nouns": "доктор|врач|медик|терапевт|бред|лекарь|doctor|medic|physician",
    },
    "architect": {
        "phrases": """
архитектор|хаос|архитектор хаоса|разработчик|программист|пишет код|ядро
операционная система|создатель|строит миры|ассемблер|баги|компилятор|раст
кодер|пишет ос|разраб|создал этот мир|автор системы|делает ос|сишник|хакер
строит хаос|повелитель хаоса|кодит по ночам|пишет на расте|голое железо|прерывания
architect|chaos|architect of chaos|developer|programmer|writes code|kernel
creator|builds worlds|assembly|bugs|compiler|rust|coder|hacker|os dev|bare metal
""",
        "nouns": "архитектор|разработчик|программист|создатель|кодер|хакер|developer|coder|architect",
    },
}

HINT_PREFIX_RU = [
    "",
    "это ",
    "он ",
    "мне кажется ",
    "думаю это ",
    "подсказка ",
    "кажется ",
    "по-моему ",
    "может это ",
    "загадал ",
    "кто-то типа ",
    "это вроде ",
    "намекаю ",
]
HINT_PREFIX_EN = ["", "it's ", "he is ", "i think ", "hint ", "maybe it's ", "someone like ", "kind of a "]

NEG_TEMPLATES_RU = ["не {n}", "это не {n}", "он не {n}", "точно не {n}", "вообще не {n}"]
NEG_TEMPLATES_EN = ["not a {n}", "not {n}", "he is not a {n}", "it's not a {n}"]

FILLERS_RU = ["ну", "эм", "хм", "ммм", "слушай", "короче", "так"]
FILLERS_EN = ["well", "hmm", "um", "uh", "so"]
PUNCT = ["", "", "", "!", "?", "...", ".", "!!", ")"]

# Соседние клавиши для «опечаток» (ЙЦУКЕН и QWERTY)
_ROWS = ["йцукенгшщзхъ", "фывапролджэ", "ячсмитьбю", "qwertyuiop", "asdfghjkl", "zxcvbnm"]
NEIGHBORS: dict[str, str] = {}
for row in _ROWS:
    for i, ch in enumerate(row):
        NEIGHBORS[ch] = row[max(0, i - 1) : i] + row[i + 1 : i + 2]


def _split(block: str) -> list[str]:
    out = []
    for part in block.replace("\n", "|").split("|"):
        part = part.strip()
        if part:
            out.append(part)
    return out


def _is_ru(text: str) -> bool:
    return any("Ѐ" <= ch <= "ӿ" for ch in text)


def _typo(text: str, rng: random.Random) -> str:
    if len(text) < 3:
        return text
    i = rng.randrange(len(text))
    op = rng.randrange(4)
    if op == 0:  # пропуск
        return text[:i] + text[i + 1 :]
    if op == 1:  # дубль
        return text[:i] + text[i] + text[i:]
    if op == 2 and i + 1 < len(text):  # перестановка
        return text[:i] + text[i + 1] + text[i] + text[i + 2 :]
    near = NEIGHBORS.get(text[i].lower())
    if near:
        return text[:i] + rng.choice(near) + text[i + 1 :]
    return text


def _stretch(text: str, rng: random.Random) -> str:
    """«дааа», «нееет», «yesss» — тянем последнюю гласную."""
    vowels = "аеиоуыэюяaeiouy"
    idx = [i for i, ch in enumerate(text) if ch in vowels]
    if not idx:
        return text
    i = idx[-1]
    return text[: i + 1] + text[i] * rng.randint(1, 3) + text[i + 1 :]


def augment(text: str, rng: random.Random, strength: float = 1.0) -> str:
    t = text
    ru = _is_ru(t)
    if rng.random() < 0.25 * strength:
        filler = rng.choice(FILLERS_RU if ru else FILLERS_EN)
        t = f"{filler} {t}" if rng.random() < 0.7 else f"{t} {filler}"
    if rng.random() < 0.15 * strength:
        t = _stretch(t, rng)
    if rng.random() < 0.2 * strength:
        t = _typo(t, rng)
    t = t + rng.choice(PUNCT)
    r = rng.random()
    if r < 0.15:
        t = t.capitalize()
    elif r < 0.2:
        t = t.upper()
    return t


def _onehot(n: int, idx: int) -> list[float]:
    v = [0.0] * n
    v[idx] = 1.0
    return v


def _intent_vec(weights: dict[str, float]) -> list[float]:
    v = [0.0] * len(INTENTS)
    for k, w in weights.items():
        v[I[k]] = w
    return v


def base_items() -> list[tuple[str, list[float], list[float] | None, str]]:
    """(text, intent_target, persona_target|None, group) — без аугментаций."""
    items = []
    for name, block in [("yes", YES), ("no", NO), ("maybe", MAYBE), ("why", WHY),
                        ("restart", RESTART), ("stop", STOP), ("greet", GREET)]:
        for phrase in _split("|".join(block)):
            items.append((phrase, _onehot(len(INTENTS), I[name]), None, name))
    for phrase, w in SOFT:
        items.append((phrase, _intent_vec(w), None, "soft"))

    hint_t = _onehot(len(INTENTS), I["hint"])
    for persona, spec in HINTS.items():
        pid = P[persona]
        for phrase in _split(spec["phrases"]):
            items.append((phrase, hint_t, _onehot(len(PERSONAS), pid), f"hint:{persona}"))
        others = [1.0 / (len(PERSONAS) - 1)] * len(PERSONAS)
        others[pid] = 0.0
        for noun in _split(spec["nouns"]):
            templates = NEG_TEMPLATES_RU if _is_ru(noun) else NEG_TEMPLATES_EN
            for tpl in templates:
                items.append((tpl.format(n=noun), hint_t, list(others), f"neg:{persona}"))
    return items


def expand(items, rng: random.Random, copies: int) -> list[tuple[str, list[float], list[float] | None]]:
    """Аугментирует базовые фразы. Подсказки дополнительно оборачиваются префиксами."""
    out = []
    for text, it, pt, group in items:
        out.append((text, it, pt))
        for _ in range(copies):
            t = text
            if group.startswith("hint:"):
                prefixes = HINT_PREFIX_RU if _is_ru(text) else HINT_PREFIX_EN
                t = rng.choice(prefixes) + t
            out.append((augment(t, rng), it, pt))
    return out


def split_items(items, rng: random.Random, holdout: float):
    """Холдаут на уровне базовых фраз — чтобы мерить обобщение, а не память."""
    train, valid = [], []
    for item in items:
        (valid if rng.random() < holdout else train).append(item)
    return train, valid
