// ============================================================
// GGUF v3 — минимальный читатель для no_std ядра
// ============================================================
// Фаза 7.1 роадмапа: «Парсер GGUF — загрузка нейросетевых весов
// на bare-metal». Ни одной аллокации: всё — срезы поверх исходного
// буфера (обычно include_bytes!, позже — файл из NSFS / serial).
//
// Формат (little-endian):
//   magic "GGUF" | version u32 | tensor_count u64 | kv_count u64
//   kv[]     : key(string) · value_type u32 · value
//   tensors[]: name(string) · n_dims u32 · dims u64[n] · ggml_type u32 · offset u64
//   padding до general.alignment (по умолчанию 32)
//   tensor data (offset — относительно начала этого блока)
//
// Всё проверяется по границам: битый файл даёт GgufError, а не #PF.
// ============================================================

#![allow(dead_code)]

pub const GGUF_MAGIC: [u8; 4] = *b"GGUF";
pub const GGUF_VERSION: u32 = 3;
const DEFAULT_ALIGNMENT: u64 = 32;
pub const MAX_DIMS: usize = 4;
const MAX_ARRAY_DEPTH: u32 = 2;

/// gguf_metadata_value_type
pub mod vtype {
    pub const UINT8: u32 = 0;
    pub const INT8: u32 = 1;
    pub const UINT16: u32 = 2;
    pub const INT16: u32 = 3;
    pub const UINT32: u32 = 4;
    pub const INT32: u32 = 5;
    pub const FLOAT32: u32 = 6;
    pub const BOOL: u32 = 7;
    pub const STRING: u32 = 8;
    pub const ARRAY: u32 = 9;
    pub const UINT64: u32 = 10;
    pub const INT64: u32 = 11;
    pub const FLOAT64: u32 = 12;
}

/// ggml_type — только те, чей размер элемента фиксирован.
/// Блочные квантования (Q4_0, Q8_0, …) пока не поддерживаются.
pub mod ggml {
    pub const F32: u32 = 0;
    pub const F16: u32 = 1;
    pub const I8: u32 = 24;
    pub const I16: u32 = 25;
    pub const I32: u32 = 26;
    pub const I64: u32 = 27;
    pub const F64: u32 = 28;
    pub const BF16: u32 = 30;

    pub fn element_size(t: u32) -> Option<u64> {
        match t {
            I8 => Some(1),
            F16 | I16 | BF16 => Some(2),
            F32 | I32 => Some(4),
            I64 | F64 => Some(8),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GgufError {
    Truncated,
    BadMagic,
    UnsupportedVersion(u32),
    BadValueType(u32),
    BadUtf8,
    TooManyDims(u32),
    BadAlignment,
    NestingTooDeep,
}

impl GgufError {
    pub fn describe(self) -> &'static str {
        match self {
            GgufError::Truncated => "file truncated",
            GgufError::BadMagic => "bad magic (not GGUF)",
            GgufError::UnsupportedVersion(_) => "unsupported GGUF version",
            GgufError::BadValueType(_) => "unknown metadata value type",
            GgufError::BadUtf8 => "string is not UTF-8",
            GgufError::TooManyDims(_) => "tensor has too many dims",
            GgufError::BadAlignment => "bad general.alignment",
            GgufError::NestingTooDeep => "metadata arrays nested too deep",
        }
    }
}

// ------------------------------------------------------------
// Курсор с проверкой границ
// ------------------------------------------------------------
#[derive(Clone, Copy)]
struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn at(buf: &'a [u8], pos: usize) -> Self {
        Self { buf, pos }
    }

    fn take(&mut self, n: u64) -> Result<&'a [u8], GgufError> {
        let n = usize::try_from(n).map_err(|_| GgufError::Truncated)?;
        let end = self.pos.checked_add(n).ok_or(GgufError::Truncated)?;
        let out = self.buf.get(self.pos..end).ok_or(GgufError::Truncated)?;
        self.pos = end;
        Ok(out)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], GgufError> {
        let mut out = [0u8; N];
        out.copy_from_slice(self.take(N as u64)?);
        Ok(out)
    }

    fn u32(&mut self) -> Result<u32, GgufError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, GgufError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn string(&mut self) -> Result<&'a str, GgufError> {
        let len = self.u64()?;
        core::str::from_utf8(self.take(len)?).map_err(|_| GgufError::BadUtf8)
    }

    fn value(&mut self, t: u32, depth: u32) -> Result<Value<'a>, GgufError> {
        Ok(match t {
            vtype::UINT8 => Value::U8(self.array::<1>()?[0]),
            vtype::INT8 => Value::I8(self.array::<1>()?[0] as i8),
            vtype::UINT16 => Value::U16(u16::from_le_bytes(self.array()?)),
            vtype::INT16 => Value::I16(i16::from_le_bytes(self.array()?)),
            vtype::UINT32 => Value::U32(self.u32()?),
            vtype::INT32 => Value::I32(i32::from_le_bytes(self.array()?)),
            vtype::FLOAT32 => Value::F32(f32::from_bits(self.u32()?)),
            vtype::BOOL => Value::Bool(self.array::<1>()?[0] != 0),
            vtype::STRING => Value::Str(self.string()?),
            vtype::UINT64 => Value::U64(self.u64()?),
            vtype::INT64 => Value::I64(i64::from_le_bytes(self.array()?)),
            vtype::FLOAT64 => Value::F64(f64::from_bits(self.u64()?)),
            vtype::ARRAY => {
                if depth >= MAX_ARRAY_DEPTH {
                    return Err(GgufError::NestingTooDeep);
                }
                let elem_type = self.u32()?;
                let len = self.u64()?;
                let start = self.pos;
                for _ in 0..len {
                    self.value(elem_type, depth + 1)?;
                }
                Value::Array(ArrayRef {
                    buf: self.buf,
                    start,
                    elem_type,
                    len,
                })
            }
            other => return Err(GgufError::BadValueType(other)),
        })
    }
}

// ------------------------------------------------------------
// Значения метаданных
// ------------------------------------------------------------
#[derive(Clone, Copy)]
pub enum Value<'a> {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    Bool(bool),
    Str(&'a str),
    Array(ArrayRef<'a>),
    U64(u64),
    I64(i64),
    F64(f64),
}

/// Ленивая ссылка на массив: элементы читаются при итерации.
/// Валидность всех элементов проверена ещё в Gguf::parse.
#[derive(Clone, Copy)]
pub struct ArrayRef<'a> {
    buf: &'a [u8],
    start: usize,
    pub elem_type: u32,
    pub len: u64,
}

impl<'a> ArrayRef<'a> {
    pub fn iter(&self) -> ArrayIter<'a> {
        ArrayIter {
            cur: Cursor::at(self.buf, self.start),
            elem_type: self.elem_type,
            left: self.len,
        }
    }
}

pub struct ArrayIter<'a> {
    cur: Cursor<'a>,
    elem_type: u32,
    left: u64,
}

impl<'a> Iterator for ArrayIter<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Value<'a>> {
        if self.left == 0 {
            return None;
        }
        self.left -= 1;
        self.cur.value(self.elem_type, 1).ok()
    }
}

// ------------------------------------------------------------
// Тензоры
// ------------------------------------------------------------
#[derive(Clone, Copy)]
pub struct Tensor<'a> {
    pub name: &'a str,
    pub ggml_type: u32,
    pub n_dims: u32,
    /// dims[0] — самая быстрая ось (как в ggml), лишние = 1
    pub dims: [u64; MAX_DIMS],
    pub data: &'a [u8],
}

impl Tensor<'_> {
    pub fn elements(&self) -> u64 {
        self.dims.iter().product()
    }
}

struct TensorInfo<'a> {
    name: &'a str,
    n_dims: u32,
    dims: [u64; MAX_DIMS],
    ggml_type: u32,
    offset: u64,
}

fn read_tensor_info<'a>(cur: &mut Cursor<'a>) -> Result<TensorInfo<'a>, GgufError> {
    let name = cur.string()?;
    let n_dims = cur.u32()?;
    if n_dims as usize > MAX_DIMS {
        return Err(GgufError::TooManyDims(n_dims));
    }
    let mut dims = [1u64; MAX_DIMS];
    for d in dims.iter_mut().take(n_dims as usize) {
        *d = cur.u64()?;
    }
    let ggml_type = cur.u32()?;
    let offset = cur.u64()?;
    Ok(TensorInfo {
        name,
        n_dims,
        dims,
        ggml_type,
        offset,
    })
}

// ------------------------------------------------------------
// Файл целиком
// ------------------------------------------------------------
pub struct Gguf<'a> {
    buf: &'a [u8],
    pub version: u32,
    pub tensor_count: u64,
    pub kv_count: u64,
    pub alignment: u64,
    kv_start: usize,
    tensor_info_start: usize,
    data_start: usize,
}

impl<'a> Gguf<'a> {
    /// Проходит весь заголовок: каждое значение и каждый tensor info
    /// проверяются на границы. После успеха get()/tensor() не паникуют.
    pub fn parse(buf: &'a [u8]) -> Result<Self, GgufError> {
        let mut cur = Cursor::at(buf, 0);
        if cur.array::<4>()? != GGUF_MAGIC {
            return Err(GgufError::BadMagic);
        }
        let version = cur.u32()?;
        if version != GGUF_VERSION {
            return Err(GgufError::UnsupportedVersion(version));
        }
        let tensor_count = cur.u64()?;
        let kv_count = cur.u64()?;

        let kv_start = cur.pos;
        let mut alignment = DEFAULT_ALIGNMENT;
        for _ in 0..kv_count {
            let key = cur.string()?;
            let t = cur.u32()?;
            let value = cur.value(t, 0)?;
            if key == "general.alignment" {
                alignment = match value {
                    Value::U32(a) => a as u64,
                    _ => return Err(GgufError::BadAlignment),
                };
            }
        }
        if alignment == 0 || !alignment.is_power_of_two() {
            return Err(GgufError::BadAlignment);
        }

        let tensor_info_start = cur.pos;
        for _ in 0..tensor_count {
            read_tensor_info(&mut cur)?;
        }

        let align = alignment as usize;
        let data_start = cur.pos.checked_add(align - 1).ok_or(GgufError::Truncated)? & !(align - 1);
        if data_start > buf.len() {
            return Err(GgufError::Truncated);
        }

        Ok(Self {
            buf,
            version,
            tensor_count,
            kv_count,
            alignment,
            kv_start,
            tensor_info_start,
            data_start,
        })
    }

    pub fn file_len(&self) -> usize {
        self.buf.len()
    }

    /// Блок тензорных данных (от выравненного начала до конца файла).
    pub fn data_region(&self) -> &'a [u8] {
        &self.buf[self.data_start..]
    }

    pub fn get(&self, key: &str) -> Option<Value<'a>> {
        let mut cur = Cursor::at(self.buf, self.kv_start);
        for _ in 0..self.kv_count {
            let k = cur.string().ok()?;
            let t = cur.u32().ok()?;
            let v = cur.value(t, 0).ok()?;
            if k == key {
                return Some(v);
            }
        }
        None
    }

    pub fn get_u32(&self, key: &str) -> Option<u32> {
        match self.get(key)? {
            Value::U32(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_f32(&self, key: &str) -> Option<f32> {
        match self.get(key)? {
            Value::F32(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&'a str> {
        match self.get(key)? {
            Value::Str(v) => Some(v),
            _ => None,
        }
    }

    pub fn get_array(&self, key: &str) -> Option<ArrayRef<'a>> {
        match self.get(key)? {
            Value::Array(v) => Some(v),
            _ => None,
        }
    }

    /// Тензор по имени. None — если нет, тип с плавающим размером
    /// (блочное квантование) или данные выходят за конец файла.
    pub fn tensor(&self, name: &str) -> Option<Tensor<'a>> {
        let mut cur = Cursor::at(self.buf, self.tensor_info_start);
        for _ in 0..self.tensor_count {
            let info = read_tensor_info(&mut cur).ok()?;
            if info.name != name {
                continue;
            }
            if info.offset % self.alignment != 0 {
                return None;
            }
            let elements = info
                .dims
                .iter()
                .try_fold(1u64, |acc, &d| acc.checked_mul(d))?;
            let bytes = elements.checked_mul(ggml::element_size(info.ggml_type)?)?;
            let start = (self.data_start as u64).checked_add(info.offset)?;
            let end = start.checked_add(bytes)?;
            let data = self.buf.get(start as usize..end as usize)?;
            return Some(Tensor {
                name: info.name,
                ggml_type: info.ggml_type,
                n_dims: info.n_dims,
                dims: info.dims,
                data,
            });
        }
        None
    }
}
