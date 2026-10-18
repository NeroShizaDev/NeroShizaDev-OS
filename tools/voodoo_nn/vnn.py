"""
vnn.py — общий контракт между тренером (Python) и ядром (core/voodoo_nn.rs).

Всё, что здесь описано, ОБЯЗАНО совпадать бит-в-бит с Rust-стороной:
  * featurizer (нормализация → слова → хэши n-грамм → бакеты)
  * целочисленный инференс (int8 веса, i32/i64 аккумуляторы, FRAC_BITS)
  * раскладка тензоров и ключи метаданных в GGUF

Если меняешь что-то тут — меняй и в core/voodoo_nn.rs, и подними
MODEL_VERSION, иначе ядро откажется грузить веса (и это правильно).
"""

from __future__ import annotations

import struct
from dataclasses import dataclass
from pathlib import Path

import numpy as np

# ------------------------------------------------------------------
# Константы модели
# ------------------------------------------------------------------
MODEL_VERSION = 1
BUCKETS = 2048
HIDDEN = 32
FRAC_BITS = 8

MAX_INPUT_CPS = 64
MAX_WORDS = 16
MAX_WORD_LEN = 24
MAX_FEATURES = 256

INTENTS = ["yes", "no", "maybe", "hint", "why", "restart", "stop", "greet"]
PERSONAS = ["petrovich", "kafidra", "tourist", "bred", "architect"]

FEATURIZER_ID = "fnv1a32/cp/word+char2-3+wordbigram/v1"

FNV_OFFSET = 0x811C9DC5
FNV_PRIME = 0x01000193

TAG_CHAR = 1
TAG_WORD = 2
TAG_BIGRAM = 3

PAD_LEFT = 0x5E  # '^'
PAD_RIGHT = 0x24  # '$'
SEP = 0x20

# ------------------------------------------------------------------
# Featurizer
# ------------------------------------------------------------------


def normalize_cp(cp: int) -> int:
    """Регистр, ё→е; всё, что не буква/цифра — разделитель."""
    if 0x41 <= cp <= 0x5A:  # A-Z
        return cp + 0x20
    if 0x410 <= cp <= 0x42F:  # А-Я
        cp += 0x20
    if cp in (0x401, 0x451):  # Ё ё
        return 0x435  # е
    if 0x61 <= cp <= 0x7A or 0x30 <= cp <= 0x39 or 0x430 <= cp <= 0x44F:
        return cp
    return SEP


def fnv_hash(tag: int, cps) -> int:
    h = FNV_OFFSET
    h = ((h ^ tag) * FNV_PRIME) & 0xFFFFFFFF
    for cp in cps:
        h = ((h ^ cp) * FNV_PRIME) & 0xFFFFFFFF
    return h


def split_words(text: str) -> list[list[int]]:
    cps = [normalize_cp(ord(ch)) for ch in text[:MAX_INPUT_CPS]]
    words: list[list[int]] = []
    cur: list[int] = []
    for cp in cps:
        if cp == SEP:
            if cur:
                words.append(cur)
                cur = []
            continue
        cur.append(cp)
    if cur:
        words.append(cur)
    words = words[:MAX_WORDS]
    return [w[:MAX_WORD_LEN] for w in words]


def featurize(text: str, buckets: int = BUCKETS) -> list[int]:
    """Список бакетов (с повторами) в каноническом порядке. Пустой → []."""
    words = split_words(text)
    out: list[int] = []

    def push(h: int) -> None:
        if len(out) < MAX_FEATURES:
            out.append(h % buckets)

    for w in words:
        push(fnv_hash(TAG_WORD, w))
        padded = [PAD_LEFT] + w + [PAD_RIGHT]
        for n in (2, 3):
            for i in range(len(padded) - n + 1):
                push(fnv_hash(TAG_CHAR, padded[i : i + n]))
    for a, b in zip(words, words[1:]):
        push(fnv_hash(TAG_BIGRAM, a + [SEP] + b))
    return out


# ------------------------------------------------------------------
# Квантованная модель + целочисленный инференс (зеркало Rust)
# ------------------------------------------------------------------


@dataclass
class QModel:
    embed: np.ndarray  # int8  [BUCKETS][HIDDEN]
    embed_bias: np.ndarray  # int32 [HIDDEN]        (единицы s_e)
    intent_w: np.ndarray  # int8  [I][HIDDEN]
    intent_b: np.ndarray  # int32 [I]             (единицы s_e*s_i)
    persona_w: np.ndarray  # int8  [P][HIDDEN]
    persona_b: np.ndarray  # int32 [P]             (единицы s_e*s_p)
    scale_embed: float
    scale_intent: float
    scale_persona: float

    @property
    def buckets(self) -> int:
        return int(self.embed.shape[0])

    @property
    def hidden(self) -> int:
        return int(self.embed.shape[1])


def trunc_div(a: int, b: int) -> int:
    """Деление с усечением к нулю — как `/` для i32 в Rust."""
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b > 0) else -q


def infer_raw(m: QModel, text: str):
    """→ (n_features, intent_acc[I], persona_acc[P]) или None. Только целые."""
    feats = featurize(text, m.buckets)
    if not feats:
        return None
    n = len(feats)
    hidden = []
    for j in range(m.hidden):
        acc = 0
        for f in feats:
            acc += int(m.embed[f, j])
        h = trunc_div(acc << FRAC_BITS, n) + (int(m.embed_bias[j]) << FRAC_BITS)
        hidden.append(h if h > 0 else 0)

    def head(w: np.ndarray, b: np.ndarray) -> list[int]:
        res = []
        for k in range(w.shape[0]):
            acc = int(b[k]) << FRAC_BITS
            for j in range(m.hidden):
                acc += hidden[j] * int(w[k, j])
            res.append(acc)
        return res

    return n, head(m.intent_w, m.intent_b), head(m.persona_w, m.persona_b)


def softmax(xs) -> np.ndarray:
    x = np.asarray(xs, dtype=np.float64)
    x = x - x.max()
    e = np.exp(x)
    return e / e.sum()


def infer_probs(m: QModel, text: str):
    raw = infer_raw(m, text)
    if raw is None:
        return None
    _, ia, pa = raw
    k = 1.0 / (1 << FRAC_BITS)
    il = [a * m.scale_embed * m.scale_intent * k for a in ia]
    pl = [a * m.scale_embed * m.scale_persona * k for a in pa]
    return softmax(il), softmax(pl)


# ------------------------------------------------------------------
# GGUF v3 (минимальный writer/reader, только нужные нам типы)
# ------------------------------------------------------------------
GGUF_MAGIC = b"GGUF"
GGUF_VERSION = 3
ALIGNMENT = 32

# gguf_metadata_value_type
T_UINT8, T_INT8, T_UINT16, T_INT16, T_UINT32, T_INT32, T_FLOAT32 = 0, 1, 2, 3, 4, 5, 6
T_BOOL, T_STRING, T_ARRAY, T_UINT64, T_INT64, T_FLOAT64 = 7, 8, 9, 10, 11, 12

# ggml_type
GGML_I8 = 24
GGML_I32 = 26

_SCALAR_FMT = {
    T_UINT8: "<B",
    T_INT8: "<b",
    T_UINT16: "<H",
    T_INT16: "<h",
    T_UINT32: "<I",
    T_INT32: "<i",
    T_FLOAT32: "<f",
    T_BOOL: "<?",
    T_UINT64: "<Q",
    T_INT64: "<q",
    T_FLOAT64: "<d",
}


def _gguf_str(s: str) -> bytes:
    b = s.encode("utf-8")
    return struct.pack("<Q", len(b)) + b


def fnv1a_bytes(data: bytes) -> int:
    h = FNV_OFFSET
    for b in data:
        h = ((h ^ b) * FNV_PRIME) & 0xFFFFFFFF
    return h


def write_gguf(path: Path, m: QModel, name: str) -> bytes:
    tensors = [
        ("embed.weight", GGML_I8, m.embed.astype(np.int8)),
        ("embed.bias", GGML_I32, m.embed_bias.astype("<i4")),
        ("intent.weight", GGML_I8, m.intent_w.astype(np.int8)),
        ("intent.bias", GGML_I32, m.intent_b.astype("<i4")),
        ("persona.weight", GGML_I8, m.persona_w.astype(np.int8)),
        ("persona.bias", GGML_I32, m.persona_b.astype("<i4")),
    ]

    # Блок данных: каждый тензор выровнен на ALIGNMENT относительно начала блока
    data = bytearray()
    infos = []
    for tname, ttype, arr in tensors:
        pad = (-len(data)) % ALIGNMENT
        data += b"\x00" * pad
        offset = len(data)
        data += arr.tobytes(order="C")
        # GGUF: ne[0] — самая быстрая ось, поэтому размеры в обратном порядке
        dims = list(reversed(arr.shape))
        infos.append((tname, dims, ttype, offset))
    data += b"\x00" * ((-len(data)) % ALIGNMENT)
    checksum = fnv1a_bytes(bytes(data))

    kvs = [
        ("general.architecture", T_STRING, "voodoo-bag"),
        ("general.name", T_STRING, name),
        ("general.alignment", T_UINT32, ALIGNMENT),
        ("voodoo.version", T_UINT32, MODEL_VERSION),
        ("voodoo.buckets", T_UINT32, m.buckets),
        ("voodoo.hidden", T_UINT32, m.hidden),
        ("voodoo.frac_bits", T_UINT32, FRAC_BITS),
        ("voodoo.featurizer", T_STRING, FEATURIZER_ID),
        ("voodoo.scale.embed", T_FLOAT32, m.scale_embed),
        ("voodoo.scale.intent", T_FLOAT32, m.scale_intent),
        ("voodoo.scale.persona", T_FLOAT32, m.scale_persona),
        ("voodoo.intent.labels", T_ARRAY, (T_STRING, INTENTS)),
        ("voodoo.persona.labels", T_ARRAY, (T_STRING, PERSONAS)),
        ("voodoo.checksum", T_UINT32, checksum),
    ]

    out = bytearray()
    out += GGUF_MAGIC
    out += struct.pack("<I", GGUF_VERSION)
    out += struct.pack("<Q", len(tensors))
    out += struct.pack("<Q", len(kvs))
    for key, vtype, val in kvs:
        out += _gguf_str(key)
        out += struct.pack("<I", vtype)
        if vtype == T_STRING:
            out += _gguf_str(val)
        elif vtype == T_ARRAY:
            etype, items = val
            out += struct.pack("<I", etype)
            out += struct.pack("<Q", len(items))
            for it in items:
                out += _gguf_str(it) if etype == T_STRING else struct.pack(_SCALAR_FMT[etype], it)
        else:
            out += struct.pack(_SCALAR_FMT[vtype], val)
    for tname, dims, ttype, offset in infos:
        out += _gguf_str(tname)
        out += struct.pack("<I", len(dims))
        for d in dims:
            out += struct.pack("<Q", d)
        out += struct.pack("<I", ttype)
        out += struct.pack("<Q", offset)
    out += b"\x00" * ((-len(out)) % ALIGNMENT)
    out += data

    path.write_bytes(bytes(out))
    return bytes(out)


class _Cur:
    def __init__(self, buf: bytes):
        self.buf = buf
        self.pos = 0

    def take(self, n: int) -> bytes:
        if self.pos + n > len(self.buf):
            raise ValueError("GGUF: unexpected EOF")
        b = self.buf[self.pos : self.pos + n]
        self.pos += n
        return b

    def unpack(self, fmt: str):
        return struct.unpack(fmt, self.take(struct.calcsize(fmt)))[0]

    def string(self) -> str:
        return self.take(self.unpack("<Q")).decode("utf-8")

    def value(self, vtype: int):
        if vtype == T_STRING:
            return self.string()
        if vtype == T_ARRAY:
            etype = self.unpack("<I")
            n = self.unpack("<Q")
            return [self.value(etype) for _ in range(n)]
        return self.unpack(_SCALAR_FMT[vtype])


def read_gguf(path: Path) -> tuple[dict, QModel]:
    buf = path.read_bytes()
    c = _Cur(buf)
    if c.take(4) != GGUF_MAGIC:
        raise ValueError("not a GGUF file")
    if c.unpack("<I") != GGUF_VERSION:
        raise ValueError("unsupported GGUF version")
    n_tensors = c.unpack("<Q")
    n_kv = c.unpack("<Q")
    kv = {}
    for _ in range(n_kv):
        key = c.string()
        kv[key] = c.value(c.unpack("<I"))
    infos = {}
    for _ in range(n_tensors):
        name = c.string()
        nd = c.unpack("<I")
        dims = [c.unpack("<Q") for _ in range(nd)]
        ttype = c.unpack("<I")
        off = c.unpack("<Q")
        infos[name] = (dims, ttype, off)
    align = kv.get("general.alignment", ALIGNMENT)
    data_start = c.pos + ((-c.pos) % align)
    data = buf[data_start:]
    if fnv1a_bytes(data) != kv["voodoo.checksum"]:
        raise ValueError("GGUF: checksum mismatch")

    def tensor(name: str) -> np.ndarray:
        dims, ttype, off = infos[name]
        dt = {GGML_I8: np.int8, GGML_I32: np.dtype("<i4")}[ttype]
        count = int(np.prod(dims))
        arr = np.frombuffer(data, dtype=dt, count=count, offset=off)
        return arr.reshape(list(reversed(dims)))

    m = QModel(
        embed=tensor("embed.weight"),
        embed_bias=tensor("embed.bias"),
        intent_w=tensor("intent.weight"),
        intent_b=tensor("intent.bias"),
        persona_w=tensor("persona.weight"),
        persona_b=tensor("persona.bias"),
        scale_embed=kv["voodoo.scale.embed"],
        scale_intent=kv["voodoo.scale.intent"],
        scale_persona=kv["voodoo.scale.persona"],
    )
    return kv, m
