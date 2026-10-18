#!/usr/bin/env python3
"""
train.py — обучает нейро-слой для VoodooEngine и кладёт веса в ядро.

    python tools/voodoo_nn/train.py            # обучить + экспорт
    python tools/voodoo_nn/train.py --check    # только проверить готовый .gguf

Архитектура (fastText-подобная, влезает в 70 КБ):

    текст ─► хэши n-грамм (2048 бакетов)
          ─► EmbeddingBag(avg, 32) + bias ─► ReLU
          ─┬► intent  head (8)  : да/нет/хз/подсказка/почему/заново/стоп/привет
           └► persona head (5)  : про кого подсказка

Экспорт: int8 веса + i32 биасы в GGUF v3 (apps/voodoo/weights/oracle.gguf),
golden-векторы для `voodoo test` в ядре (apps/voodoo/golden.rs).
Ядро считает тот же инференс в целых числах и сверяет аккумуляторы бит-в-бит.
"""

from __future__ import annotations

import argparse
import random
import sys
from pathlib import Path

import numpy as np

import corpus
import vnn

ROOT = Path(__file__).resolve().parents[2]
GGUF_PATH = ROOT / "apps" / "voodoo" / "weights" / "oracle.gguf"
GOLDEN_PATH = ROOT / "apps" / "voodoo" / "golden.rs"

GOLDEN_TEXTS = [
    "да",
    "нет",
    "хз",
    "скорее да",
    "конечно нет",
    "почему?",
    "стоп",
    "заново",
    "привет",
    "король жижи",
    "он врач",
    "любит бананы",
    "архитектор хаоса",
    "это группа учёных",
    "не доктор",
    "yes",
    "nope",
    "not sure",
    "banana tourist",
    "Ну ДААА!!!",
]


# ------------------------------------------------------------------
# Обучение (float32, numpy)
# ------------------------------------------------------------------
class Net:
    def __init__(self, rng: np.random.Generator):
        h, b = vnn.HIDDEN, vnn.BUCKETS
        ni, npn = len(vnn.INTENTS), len(vnn.PERSONAS)
        self.p = {
            "E": rng.normal(0, 0.1, (b, h)).astype(np.float32),
            "b1": np.full(h, 0.05, np.float32),
            "Wi": rng.normal(0, 1 / np.sqrt(h), (ni, h)).astype(np.float32),
            "bi": np.zeros(ni, np.float32),
            "Wp": rng.normal(0, 1 / np.sqrt(h), (npn, h)).astype(np.float32),
            "bp": np.zeros(npn, np.float32),
        }
        self.m = {k: np.zeros_like(v) for k, v in self.p.items()}
        self.v = {k: np.zeros_like(v) for k, v in self.p.items()}
        self.t = 0

    def forward(self, A):
        p = self.p
        z = A @ p["E"] + p["b1"]
        h = np.maximum(z, 0)
        return z, h, h @ p["Wi"].T + p["bi"], h @ p["Wp"].T + p["bp"]

    def step(self, A, Ti, Tp, mask, lr, wd=1e-5, persona_w=1.0):
        p = self.p
        n = A.shape[0]
        z, h, li, lp = self.forward(A)
        si = _softmax(li)
        sp = _softmax(lp)
        msum = max(1.0, float(mask.sum()))
        loss = -(Ti * np.log(si + 1e-9)).sum() / n
        loss += persona_w * -(mask[:, None] * Tp * np.log(sp + 1e-9)).sum() / msum

        dli = (si - Ti) / n
        dlp = persona_w * (sp - Tp) * mask[:, None] / msum
        g = {
            "Wi": dli.T @ h,
            "bi": dli.sum(0),
            "Wp": dlp.T @ h,
            "bp": dlp.sum(0),
        }
        dz = (dli @ p["Wi"] + dlp @ p["Wp"]) * (z > 0)
        g["b1"] = dz.sum(0)
        g["E"] = A.T @ dz

        self.t += 1
        b1, b2 = 0.9, 0.999
        for k in p:
            gk = g[k] + wd * p[k]
            self.m[k] = b1 * self.m[k] + (1 - b1) * gk
            self.v[k] = b2 * self.v[k] + (1 - b2) * gk * gk
            mh = self.m[k] / (1 - b1**self.t)
            vh = self.v[k] / (1 - b2**self.t)
            p[k] -= lr * mh / (np.sqrt(vh) + 1e-8)
        return loss


def _softmax(x):
    x = x - x.max(axis=1, keepdims=True)
    e = np.exp(x)
    return e / e.sum(axis=1, keepdims=True)


def bag_matrix(feats_list, rng: np.random.Generator | None, drop: float):
    A = np.zeros((len(feats_list), vnn.BUCKETS), np.float32)
    for r, feats in enumerate(feats_list):
        if not feats:
            continue
        f = np.asarray(feats)
        if rng is not None and drop > 0 and len(f) > 2:
            keep = rng.random(len(f)) >= drop
            if keep.any():
                f = f[keep]
        np.add.at(A[r], f, 1.0)
        A[r] /= len(f)
    return A


def encode(samples):
    feats = [vnn.featurize(t) for t, _, _ in samples]
    Ti = np.asarray([it for _, it, _ in samples], np.float32)
    Ti /= Ti.sum(axis=1, keepdims=True)
    Tp = np.zeros((len(samples), len(vnn.PERSONAS)), np.float32)
    mask = np.zeros(len(samples), np.float32)
    for r, (_, _, pt) in enumerate(samples):
        if pt is not None:
            Tp[r] = pt
            mask[r] = 1.0
    return feats, Ti, Tp, mask


def accuracy_float(net: Net, feats, Ti, Tp, mask):
    A = bag_matrix(feats, None, 0.0)
    _, _, li, lp = net.forward(A)
    ia = float((li.argmax(1) == Ti.argmax(1)).mean())
    hard = (mask > 0) & (Tp.max(1) > 0.99)
    pa = float((lp.argmax(1)[hard] == Tp.argmax(1)[hard]).mean()) if hard.any() else float("nan")
    return ia, pa


# ------------------------------------------------------------------
# Квантование
# ------------------------------------------------------------------
def quantize(net: Net) -> vnn.QModel:
    p = net.p

    def sym_scale(w):
        return float(np.abs(w).max()) / 127.0 or 1.0

    # f32-округление масштабов ДО квантования — ядро прочитает ровно эти f32
    se = float(np.float32(sym_scale(p["E"])))
    si = float(np.float32(sym_scale(p["Wi"])))
    sp = float(np.float32(sym_scale(p["Wp"])))

    def q8(w, s):
        return np.clip(np.rint(w / s), -127, 127).astype(np.int8)

    def q32(b, s):
        return np.rint(b / s).astype(np.int64).clip(-(2**31), 2**31 - 1).astype(np.int32)

    return vnn.QModel(
        embed=q8(p["E"], se),
        embed_bias=q32(p["b1"], se),
        intent_w=q8(p["Wi"], si),
        intent_b=q32(p["bi"], se * si),
        persona_w=q8(p["Wp"], sp),
        persona_b=q32(p["bp"], se * sp),
        scale_embed=se,
        scale_intent=si,
        scale_persona=sp,
    )


def accuracy_quant(m: vnn.QModel, samples):
    ok_i = tot_i = ok_p = tot_p = 0
    for text, it, pt in samples:
        res = vnn.infer_probs(m, text)
        if res is None:
            continue
        pi, pp = res
        tot_i += 1
        ok_i += int(np.argmax(pi) == int(np.argmax(it)))
        if pt is not None and max(pt) > 0.99:
            tot_p += 1
            ok_p += int(np.argmax(pp) == int(np.argmax(pt)))
    return ok_i / max(1, tot_i), ok_p / max(1, tot_p)


# ------------------------------------------------------------------
# Golden-векторы для ядра
# ------------------------------------------------------------------
def rust_str(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def write_golden(m: vnn.QModel, path: Path) -> None:
    lines = [
        "// @generated by tools/voodoo_nn/train.py — не править руками.",
        "//",
        "// Golden-векторы: фраза → точные целочисленные аккумуляторы логитов,",
        "// посчитанные Python-зеркалом (vnn.infer_raw) по тому же oracle.gguf.",
        "// `voodoo test` в ядре обязан получить те же числа бит-в-бит.",
        "",
        "pub struct Golden {",
        "    pub text: &'static str,",
        "    pub features: u16,",
        f"    pub intent_acc: [i64; {len(vnn.INTENTS)}],",
        f"    pub persona_acc: [i64; {len(vnn.PERSONAS)}],",
        "}",
        "",
        "#[rustfmt::skip]",
        "pub const GOLDEN: &[Golden] = &[",
    ]
    for text in GOLDEN_TEXTS:
        n, ia, pa = vnn.infer_raw(m, text)
        lines += [
            "    Golden {",
            f"        text: {rust_str(text)},",
            f"        features: {n},",
            f"        intent_acc: [{', '.join(str(x) for x in ia)}],",
            f"        persona_acc: [{', '.join(str(x) for x in pa)}],",
            "    },",
        ]
    lines += ["];", ""]
    path.write_text("\n".join(lines), encoding="utf-8", newline="\n")


def show_examples(m: vnn.QModel) -> None:
    print("\n[демо] что видит нейро-слой:")
    for text in GOLDEN_TEXTS:
        pi, pp = vnn.infer_probs(m, text)
        top = int(np.argmax(pi))
        extra = ""
        if vnn.INTENTS[top] == "hint":
            pk = int(np.argmax(pp))
            extra = f"  → {vnn.PERSONAS[pk]} {pp[pk] * 100:4.0f}%"
        print(f"  {text!r:24} {vnn.INTENTS[top]:8} {pi[top] * 100:4.0f}%{extra}")


# ------------------------------------------------------------------
# Независимая проверка через пакет `gguf` (если установлен)
# ------------------------------------------------------------------
def check_with_reference_reader(path: Path) -> None:
    try:
        import types

        sys.modules.setdefault("yaml", types.ModuleType("yaml"))  # gguf тянет yaml ради метаданных
        from gguf import GGUFReader  # type: ignore
    except Exception as exc:  # noqa: BLE001
        print(f"[gguf] эталонный reader недоступен ({exc.__class__.__name__}) — пропуск")
        return
    reader = GGUFReader(str(path))
    names = [t.name for t in reader.tensors]
    shapes = {t.name: [int(x) for x in t.shape] for t in reader.tensors}
    print(f"[gguf] эталонный reader: {len(reader.fields)} полей, тензоры {names}")
    print(f"[gguf] формы: {shapes}")


def check(path: Path) -> vnn.QModel:
    kv, m = vnn.read_gguf(path)
    print(f"[check] {path.relative_to(ROOT)}: {path.stat().st_size} байт, "
          f"{kv['general.name']!r}, buckets={m.buckets} hidden={m.hidden}")
    check_with_reference_reader(path)
    return m


def fit(samples, epochs: int, rng_np: np.random.Generator, valid) -> Net:
    tr = encode(samples)
    va = encode(valid) if valid else None
    net = Net(rng_np)
    batch = 64
    n = len(samples)
    for epoch in range(epochs):
        lr = 0.01 * (0.5 ** (epoch / 15))
        order = rng_np.permutation(n)
        total = 0.0
        for s in range(0, n, batch):
            idx = order[s : s + batch]
            A = bag_matrix([tr[0][i] for i in idx], rng_np, drop=0.15)
            total += net.step(A, tr[1][idx], tr[2][idx], tr[3][idx], lr) * len(idx)
        if epoch % 10 == 9 or epoch == epochs - 1:
            ti, tp = accuracy_float(net, *tr)
            line = f"[ep {epoch + 1:3}] loss {total / n:.4f}  train intent {ti:.3f} persona {tp:.3f}"
            if va is not None:
                vi, vp = accuracy_float(net, *va)
                line += f"  | holdout intent {vi:.3f} persona {vp:.3f}"
            print(line)
    return net


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--epochs", type=int, default=40)
    ap.add_argument("--seed", type=int, default=1337)
    ap.add_argument("--copies", type=int, default=14)
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--skip-eval", action="store_true", help="не гонять проход с холдаутом")
    args = ap.parse_args()

    if args.check:
        m = check(GGUF_PATH)
        show_examples(m)
        return 0

    rng_py = random.Random(args.seed)
    rng_np = np.random.default_rng(args.seed)

    items = corpus.base_items()
    train_items, valid_items = corpus.split_items(items, rng_py, holdout=0.12)
    print(f"[data] базовых фраз: {len(items)} (train {len(train_items)}, holdout {len(valid_items)})")

    # Проход 1: честная метрика на фразах, которых сеть не видела ни в каком виде
    if not args.skip_eval:
        valid = corpus.expand(valid_items, rng_py, 4)
        net = fit(corpus.expand(train_items, rng_py, args.copies), args.epochs, rng_np, valid)
        qi, qp = accuracy_quant(quantize(net), valid)
        print(f"[int8] holdout после квантования: intent {qi:.3f} persona {qp:.3f}")

    # Проход 2: боевые веса — на всём корпусе. Свои генераторы, чтобы веса
    # не зависели от того, гоняли ли проход 1 (воспроизводимость бит-в-бит).
    rng_py = random.Random(args.seed + 1)
    rng_np = np.random.default_rng(args.seed + 1)
    full = corpus.expand(items, rng_py, args.copies)
    print(f"[final] обучение на всём корпусе: {len(full)} примеров")
    net = fit(full, args.epochs, rng_np, None)
    qm = quantize(net)
    qi, qp = accuracy_quant(qm, full)
    print(f"[int8] финальная модель на корпусе: intent {qi:.3f} persona {qp:.3f}")

    GGUF_PATH.parent.mkdir(parents=True, exist_ok=True)
    vnn.write_gguf(GGUF_PATH, qm, "NeroShiza Voodoo Oracle")
    m = check(GGUF_PATH)

    # Round-trip: то, что прочитали из файла, обязано совпасть с тем, что квантовали
    for text in GOLDEN_TEXTS:
        assert vnn.infer_raw(m, text) == vnn.infer_raw(qm, text), text
    write_golden(m, GOLDEN_PATH)
    print(f"[out] {GOLDEN_PATH.relative_to(ROOT)}: {len(GOLDEN_TEXTS)} golden-векторов")
    show_examples(m)
    return 0


if __name__ == "__main__":
    sys.exit(main())
