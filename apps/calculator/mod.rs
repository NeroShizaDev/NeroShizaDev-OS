//! NeroShiza Engineering Calculator — полное соответствие ISO/IEC 60559:2020
//! no_std, f64, x87 80-bit, ручной ввод/вывод, без аллокатора.

#![allow(dead_code, unused_unsafe)]

use crate::apps::games::common_hw::GameInput;
use crate::locale;
use core::arch::asm;

const PI: f64 = core::f64::consts::PI;

fn trunc_f64(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    if x >= 0.0 {
        (x as i64) as f64
    } else {
        -((-x) as i64 as f64)
    }
}

fn fract_f64(x: f64) -> f64 {
    x - trunc_f64(x)
}

fn round_f64(x: f64) -> f64 {
    if x >= 0.0 {
        trunc_f64(x + 0.5)
    } else {
        trunc_f64(x - 0.5)
    }
}

// ==================================================================
// 1. ОПРЕДЕЛЕНИЕ ПОДДЕРЖКИ FPU / SSE / AVX (опционально)
// ==================================================================
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FpuMode {
    X87,
    SSE,
    AVX,
}

pub fn detect_fpu_mode() -> FpuMode {
    #[cfg(target_arch = "x86_64")]
    {
        let cpuid_res = unsafe { core::arch::x86_64::__cpuid(1) };
        if (cpuid_res.ecx & (1 << 28)) != 0 {
            return FpuMode::AVX;
        }
        if (cpuid_res.edx & (1 << 25)) != 0 {
            return FpuMode::SSE;
        }
    }
    FpuMode::X87
}

// ==================================================================
// 2. МАТЕМАТИЧЕСКОЕ ЯДРО (x87 80-bit, f64 память)
// ==================================================================
mod x87 {
    use super::*;

    // ---- базовая арифметика с правильным порядком операндов ----
    #[inline(always)]
    pub unsafe fn add(a: f64, b: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fadd qword ptr [{1}]", "fstp qword ptr [{2}]",
             in(reg) &a, in(reg) &b, in(reg) &mut res);
        res
    }

    #[inline(always)]
    pub unsafe fn sub(a: f64, b: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fsub qword ptr [{1}]", "fstp qword ptr [{2}]",
             in(reg) &a, in(reg) &b, in(reg) &mut res);
        res
    }

    #[inline(always)]
    pub unsafe fn mul(a: f64, b: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fmul qword ptr [{1}]", "fstp qword ptr [{2}]",
             in(reg) &a, in(reg) &b, in(reg) &mut res);
        res
    }

    #[inline(always)]
    pub unsafe fn div(a: f64, b: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fdiv qword ptr [{1}]", "fstp qword ptr [{2}]",
             in(reg) &a, in(reg) &b, in(reg) &mut res);
        res
    }

    // ---- тригонометрические и трансцендентные ----
    pub unsafe fn sin(x: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fsin", "fstp qword ptr [{1}]", in(reg) &x, in(reg) &mut res);
        res
    }
    pub unsafe fn cos(x: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fcos", "fstp qword ptr [{1}]", in(reg) &x, in(reg) &mut res);
        res
    }
    pub unsafe fn tan(x: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fptan", "fstp st(0)", "fstp qword ptr [{1}]",
             in(reg) &x, in(reg) &mut res);
        res
    }

    // asin(x) = atan2(x, sqrt(1-x²))
    pub unsafe fn asin(x: f64) -> f64 {
        let mut res = 0.0;
        asm!(
            "fld qword ptr [{0}]", "fld st(0)", "fmul st(0), st(1)", "fld1", "fsubrp", "fsqrt",
            "fpatan", "fstp qword ptr [{1}]",
            in(reg) &x, in(reg) &mut res
        );
        res
    }

    // acos(x) = π/2 - asin(x)
    pub unsafe fn acos(x: f64) -> f64 {
        PI / 2.0 - asin(x)
    }

    // atan(x) = atan2(x, 1), обрабатываем ±∞ явно
    pub unsafe fn atan(x: f64) -> f64 {
        if x.is_infinite() {
            return if x.is_sign_positive() {
                PI / 2.0
            } else {
                -PI / 2.0
            };
        }
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fld1", "fpatan", "fstp qword ptr [{1}]",
             in(reg) &x, in(reg) &mut res);
        res
    }

    pub unsafe fn sqrt(x: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld qword ptr [{0}]", "fsqrt", "fstp qword ptr [{1}]", in(reg) &x, in(reg) &mut res);
        res
    }

    // ln(x) = log2(x) * ln(2)
    pub unsafe fn ln(x: f64) -> f64 {
        let mut res = 0.0;
        asm!("fldln2", "fld qword ptr [{0}]", "fyl2x", "fstp qword ptr [{1}]",
             in(reg) &x, in(reg) &mut res);
        res
    }

    // log10(x) = log2(x) * log10(2)
    pub unsafe fn log10(x: f64) -> f64 {
        let mut res = 0.0;
        asm!("fldlg2", "fld qword ptr [{0}]", "fyl2x", "fstp qword ptr [{1}]",
             in(reg) &x, in(reg) &mut res);
        res
    }

    // log2(x) через встроенную x87 операцию y * log2(x) с y = 1
    pub unsafe fn log2(x: f64) -> f64 {
        let mut res = 0.0;
        asm!("fld1", "fld qword ptr [{0}]", "fyl2x", "fstp qword ptr [{1}]",
             in(reg) &x, in(reg) &mut res);
        res
    }

    // exp(x) = 2^(x * log2(e))
    pub unsafe fn exp(x: f64) -> f64 {
        let mut res = 0.0;
        asm!(
            "fldl2e", "fld qword ptr [{0}]", "fmulp", "fld st(0)", "frndint", "fsub st(1), st",
            "fxch", "f2xm1", "fld1", "faddp", "fscale", "fstp st(1)", "fstp qword ptr [{1}]",
            in(reg) &x, in(reg) &mut res
        );
        res
    }

    // x^y = 2^(y * log2(x))
    pub unsafe fn pow(x: f64, y: f64) -> f64 {
        let mut res = 0.0;
        asm!(
            "fld qword ptr [{1}]", "fld qword ptr [{0}]", "fyl2x", "fld st(0)", "frndint",
            "fsub st(1), st", "fxch", "f2xm1", "fld1", "faddp", "fscale", "fstp st(1)",
            "fstp qword ptr [{2}]",
            in(reg) &x, in(reg) &y, in(reg) &mut res
        );
        res
    }

    // Гамма-функция Ланцоша (g=7)
    pub unsafe fn gamma(z: f64) -> f64 {
        const P: [f64; 9] = [
            0.99999999999980993,
            676.5203681218851,
            -1259.1392167224028,
            771.32342877765313,
            -176.61502916214059,
            12.507343278686905,
            -0.13857109526572012,
            9.984369578019572e-6,
            1.5056327351493116e-7,
        ];
        if z < 0.5 {
            let pi = core::f64::consts::PI;
            pi / (sin(pi * z) * gamma(1.0 - z))
        } else {
            let z = z - 1.0;
            let mut x = P[0];
            for i in 1..9 {
                x += P[i] / (z + i as f64);
            }
            let t = z + 7.5;
            pow(t, z + 0.5) * exp(-t) * x
        }
    }

    pub unsafe fn factorial(n: f64) -> f64 {
        if n < 0.0 {
            return core::f64::NAN;
        }
        if fract_f64(n).abs() < 1e-12 && n <= 170.0 {
            let mut res = 1.0;
            for i in 2..=(n as u64) {
                res *= i as f64;
            }
            res
        } else if n > 170.0 {
            core::f64::INFINITY
        } else {
            gamma(n + 1.0)
        }
    }
}

// ==================================================================
// 3. ОПЕРАТОРЫ
// ==================================================================
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    PercentOf,
    PercentTo,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MathMode {
    Ieee754_2008,
    Ieee754_2019,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InvalidKind {
    ZeroDivZero,
    InfDivInf,
    InfMinusInf,
    ZeroTimesInf,
    SqrtNegative,
    LogNegative,
    PowZeroZero,
    PowOneInf,
    NegBaseFracPow,
    NonIntegerArg,
    RootZero,
    UnsupportedInMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalcStatus {
    pub indeterminate: bool,
    pub invalid: Option<InvalidKind>,
}

impl CalcStatus {
    pub const fn new() -> Self {
        Self {
            indeterminate: false,
            invalid: None,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn mark_indeterminate(&mut self, kind: InvalidKind) {
        self.indeterminate = true;
        if self.invalid.is_none() {
            self.invalid = Some(kind);
        }
    }

    pub fn mark_invalid(&mut self, kind: InvalidKind) {
        if self.invalid.is_none() {
            self.invalid = Some(kind);
        }
    }
}

fn status_tag(kind: InvalidKind) -> &'static str {
    match kind {
        InvalidKind::ZeroDivZero => "NaN[0/0]",
        InvalidKind::InfDivInf => "NaN[inf/inf]",
        InvalidKind::InfMinusInf => "NaN[inf-inf]",
        InvalidKind::ZeroTimesInf => "NaN[0*inf]",
        InvalidKind::SqrtNegative => "NaN[sqrt-]",
        InvalidKind::LogNegative => "NaN[log-]",
        InvalidKind::PowZeroZero => "NaN[0^0]",
        InvalidKind::PowOneInf => "NaN[1^inf]",
        InvalidKind::NegBaseFracPow => "NaN[neg^frac]",
        InvalidKind::NonIntegerArg => "NaN[int]",
        InvalidKind::RootZero => "NaN[root0]",
        InvalidKind::UnsupportedInMode => "NaN[mode]",
    }
}

fn is_near_integer(x: f64) -> Option<i64> {
    if !x.is_finite() {
        return None;
    }
    let rounded = round_f64(x);
    if (x - rounded).abs() <= 1e-12 && rounded >= i64::MIN as f64 && rounded <= i64::MAX as f64 {
        Some(rounded as i64)
    } else {
        None
    }
}

fn parity_sign(n: i64) -> f64 {
    if n.rem_euclid(2) == 0 { 1.0 } else { -1.0 }
}

fn sin_pi(x: f64) -> f64 {
    if !x.is_finite() {
        return core::f64::NAN;
    }
    if let Some(n2) = is_near_integer(x * 2.0) {
        if n2.rem_euclid(2) == 0 {
            return 0.0 * x;
        }
        return parity_sign((n2 - 1) / 2);
    }
    unsafe { x87::sin(PI * x) }
}

fn cos_pi(x: f64) -> f64 {
    if !x.is_finite() {
        return core::f64::NAN;
    }
    if let Some(n2) = is_near_integer(x * 2.0) {
        if n2.rem_euclid(2) != 0 {
            return 0.0;
        }
        return parity_sign(n2 / 2);
    }
    unsafe { x87::cos(PI * x) }
}

fn tan_pi(x: f64) -> f64 {
    if !x.is_finite() {
        return core::f64::NAN;
    }
    if let Some(n2) = is_near_integer(x * 2.0) {
        if n2.rem_euclid(2) == 0 {
            return 0.0 * x;
        }
        return if parity_sign((n2 - 1) / 2).is_sign_positive() {
            core::f64::INFINITY
        } else {
            core::f64::NEG_INFINITY
        };
    }
    unsafe { x87::tan(PI * x) }
}

fn rsqrt(x: f64) -> f64 {
    if x == 0.0 {
        return if x.is_sign_negative() {
            core::f64::NEG_INFINITY
        } else {
            core::f64::INFINITY
        };
    }
    if x < 0.0 {
        return core::f64::NAN;
    }
    unsafe { x87::div(1.0, x87::sqrt(x)) }
}

fn powr(x: f64, y: f64, status: &mut CalcStatus) -> f64 {
    if x.is_nan() || y.is_nan() {
        return core::f64::NAN;
    }
    if x < 0.0 {
        status.mark_invalid(InvalidKind::NegBaseFracPow);
        return core::f64::NAN;
    }
    if x == 0.0 && y == 0.0 {
        status.mark_indeterminate(InvalidKind::PowZeroZero);
        return core::f64::NAN;
    }
    if x == 1.0 && y.is_infinite() {
        status.mark_indeterminate(InvalidKind::PowOneInf);
        return core::f64::NAN;
    }
    unsafe { x87::pow(x, y) }
}

fn pown(x: f64, y: f64, status: &mut CalcStatus) -> f64 {
    let Some(power) = is_near_integer(y) else {
        status.mark_invalid(InvalidKind::NonIntegerArg);
        return core::f64::NAN;
    };
    if power == 0 {
        return 1.0;
    }
    if x == 0.0 && power < 0 {
        return if x.is_sign_negative() && power.rem_euclid(2) != 0 {
            core::f64::NEG_INFINITY
        } else {
            core::f64::INFINITY
        };
    }
    unsafe { x87::pow(x, power as f64) }
}

fn rootn(x: f64, y: f64, status: &mut CalcStatus) -> f64 {
    let Some(root) = is_near_integer(y) else {
        status.mark_invalid(InvalidKind::NonIntegerArg);
        return core::f64::NAN;
    };
    if root == 0 {
        status.mark_invalid(InvalidKind::RootZero);
        return core::f64::NAN;
    }
    if x == 0.0 && root < 0 {
        return if x.is_sign_negative() && root.rem_euclid(2) != 0 {
            core::f64::NEG_INFINITY
        } else {
            core::f64::INFINITY
        };
    }
    if x < 0.0 {
        if root.rem_euclid(2) == 0 {
            status.mark_invalid(InvalidKind::NegBaseFracPow);
            return core::f64::NAN;
        }
        let abs = unsafe { x87::pow(-x, 1.0 / root as f64) };
        return -abs;
    }
    unsafe { x87::pow(x, 1.0 / root as f64) }
}

impl Operator {
    pub fn precedence(self) -> u8 {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div | Operator::PercentOf | Operator::PercentTo => 2,
            Operator::Pow => 3,
        }
    }
    pub fn right_assoc(self) -> bool {
        matches!(self, Operator::Pow)
    }

    pub fn apply(self, a: f64, b: f64, math_mode: MathMode, status: &mut CalcStatus) -> f64 {
        unsafe {
            match self {
                Operator::Add => {
                    if a.is_infinite()
                        && b.is_infinite()
                        && a.is_sign_positive() != b.is_sign_positive()
                    {
                        status.mark_invalid(InvalidKind::InfMinusInf);
                        core::f64::NAN
                    } else {
                        x87::add(a, b)
                    }
                }
                Operator::Sub => {
                    if a.is_infinite()
                        && b.is_infinite()
                        && a.is_sign_positive() == b.is_sign_positive()
                    {
                        status.mark_invalid(InvalidKind::InfMinusInf);
                        core::f64::NAN
                    } else {
                        x87::sub(a, b)
                    }
                }
                Operator::Mul => {
                    if (a == 0.0 && b.is_infinite()) || (b == 0.0 && a.is_infinite()) {
                        status.mark_invalid(InvalidKind::ZeroTimesInf);
                        core::f64::NAN
                    } else {
                        x87::mul(a, b)
                    }
                }
                Operator::Div => {
                    if b == 0.0 {
                        if a == 0.0 {
                            status.mark_invalid(InvalidKind::ZeroDivZero);
                            core::f64::NAN
                        } else if b.is_sign_negative() {
                            if a.is_sign_positive() {
                                core::f64::NEG_INFINITY
                            } else {
                                core::f64::INFINITY
                            }
                        } else {
                            if a.is_sign_positive() {
                                core::f64::INFINITY
                            } else {
                                core::f64::NEG_INFINITY
                            }
                        }
                    } else if a.is_infinite() && b.is_infinite() {
                        status.mark_invalid(InvalidKind::InfDivInf);
                        core::f64::NAN
                    } else {
                        x87::div(a, b)
                    }
                }
                Operator::Pow => {
                    if b == 0.0 {
                        if a == 0.0 {
                            status.mark_indeterminate(InvalidKind::PowZeroZero);
                            return if matches!(math_mode, MathMode::Ieee754_2019) {
                                1.0
                            } else {
                                core::f64::NAN
                            };
                        }
                        return 1.0;
                    }
                    if a == 1.0 {
                        if b.is_infinite() {
                            status.mark_indeterminate(InvalidKind::PowOneInf);
                        }
                        return 1.0;
                    }
                    if a == -1.0 && b.is_infinite() {
                        return 1.0;
                    }
                    if a.is_nan() || b.is_nan() {
                        return core::f64::NAN;
                    }
                    if a == 0.0 {
                        return if b < 0.0 { core::f64::INFINITY } else { 0.0 };
                    }
                    if a == core::f64::NEG_INFINITY {
                        let odd = fract_f64(b).abs() < 1e-12 && (b as i64 % 2).abs() == 1;
                        if b > 0.0 {
                            return if odd {
                                core::f64::NEG_INFINITY
                            } else {
                                core::f64::INFINITY
                            };
                        } else {
                            return if odd { -0.0 } else { 0.0 };
                        }
                    }
                    if a < 0.0 && fract_f64(b).abs() > 1e-12 {
                        status.mark_invalid(InvalidKind::NegBaseFracPow);
                        return core::f64::NAN;
                    }
                    x87::pow(a, b)
                }
                Operator::PercentOf => x87::div(x87::mul(a, b), 100.0),
                Operator::PercentTo => x87::mul(x87::div(a, b), 100.0),
            }
        }
    }
}

// ==================================================================
// 4. ФУНКЦИИ
// ==================================================================
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Function {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sqrt,
    Ln,
    Log,
    Exp,
    Exp2m1,
    Log2p1,
    Log10p1,
    Powr,
    Pown,
    Rootn,
    SinPi,
    CosPi,
    TanPi,
    AsinPi,
    AcosPi,
    AtanPi,
    Rsqrt,
}

impl Function {
    pub fn arity(self) -> usize {
        match self {
            Function::Powr | Function::Pown | Function::Rootn => 2,
            _ => 1,
        }
    }

    pub fn apply(
        self,
        x: f64,
        angle_mode: AngleMode,
        math_mode: MathMode,
        status: &mut CalcStatus,
    ) -> f64 {
        let to_rad = |deg| deg * PI / 180.0;
        let to_deg = |rad| rad * 180.0 / PI;
        match self {
            Function::Sin => unsafe {
                x87::sin(if angle_mode == AngleMode::Deg {
                    to_rad(x)
                } else {
                    x
                })
            },
            Function::Cos => unsafe {
                x87::cos(if angle_mode == AngleMode::Deg {
                    to_rad(x)
                } else {
                    x
                })
            },
            Function::Tan => {
                if angle_mode == AngleMode::Deg {
                    let norm = x % 360.0;
                    if (norm - 90.0).abs() < 1e-10 {
                        return core::f64::INFINITY;
                    }
                    if (norm + 90.0).abs() < 1e-10 {
                        return core::f64::NEG_INFINITY;
                    }
                }
                unsafe {
                    x87::tan(if angle_mode == AngleMode::Deg {
                        to_rad(x)
                    } else {
                        x
                    })
                }
            }
            Function::Asin => {
                if x < -1.0 || x > 1.0 {
                    return core::f64::NAN;
                }
                let v = unsafe { x87::asin(x) };
                if angle_mode == AngleMode::Deg {
                    to_deg(v)
                } else {
                    v
                }
            }
            Function::Acos => {
                if x < -1.0 || x > 1.0 {
                    return core::f64::NAN;
                }
                let v = unsafe { x87::acos(x) };
                if angle_mode == AngleMode::Deg {
                    to_deg(v)
                } else {
                    v
                }
            }
            Function::Atan => {
                if x.is_infinite() {
                    let r = if x.is_sign_positive() {
                        PI / 2.0
                    } else {
                        -PI / 2.0
                    };
                    return if angle_mode == AngleMode::Deg {
                        to_deg(r)
                    } else {
                        r
                    };
                }
                let v = unsafe { x87::atan(x) };
                if angle_mode == AngleMode::Deg {
                    to_deg(v)
                } else {
                    v
                }
            }
            Function::Sqrt => {
                if x < 0.0 {
                    status.mark_invalid(InvalidKind::SqrtNegative);
                    core::f64::NAN
                } else {
                    unsafe { x87::sqrt(x) }
                }
            }
            Function::Ln => {
                if x == 0.0 {
                    core::f64::NEG_INFINITY
                } else if x < 0.0 {
                    status.mark_invalid(InvalidKind::LogNegative);
                    core::f64::NAN
                } else {
                    unsafe { x87::ln(x) }
                }
            }
            Function::Log => {
                if x == 0.0 {
                    core::f64::NEG_INFINITY
                } else if x < 0.0 {
                    status.mark_invalid(InvalidKind::LogNegative);
                    core::f64::NAN
                } else {
                    unsafe { x87::log10(x) }
                }
            }
            Function::Exp => unsafe { x87::exp(x) },
            Function::Exp2m1 => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else {
                    unsafe { x87::pow(2.0, x) - 1.0 }
                }
            }
            Function::Log2p1 => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else if x == -1.0 {
                    core::f64::NEG_INFINITY
                } else if x < -1.0 {
                    status.mark_invalid(InvalidKind::LogNegative);
                    core::f64::NAN
                } else {
                    unsafe { x87::log2(1.0 + x) }
                }
            }
            Function::Log10p1 => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else if x == -1.0 {
                    core::f64::NEG_INFINITY
                } else if x < -1.0 {
                    status.mark_invalid(InvalidKind::LogNegative);
                    core::f64::NAN
                } else {
                    unsafe { x87::log10(1.0 + x) }
                }
            }
            Function::SinPi => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else {
                    sin_pi(x)
                }
            }
            Function::CosPi => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else {
                    cos_pi(x)
                }
            }
            Function::TanPi => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else {
                    tan_pi(x)
                }
            }
            Function::AsinPi => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else if x < -1.0 || x > 1.0 {
                    core::f64::NAN
                } else {
                    unsafe { x87::asin(x) / PI }
                }
            }
            Function::AcosPi => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else if x < -1.0 || x > 1.0 {
                    core::f64::NAN
                } else {
                    unsafe { x87::acos(x) / PI }
                }
            }
            Function::AtanPi => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else {
                    unsafe { x87::atan(x) / PI }
                }
            }
            Function::Rsqrt => {
                if !matches!(math_mode, MathMode::Ieee754_2019) {
                    status.mark_invalid(InvalidKind::UnsupportedInMode);
                    core::f64::NAN
                } else if x < 0.0 && x != 0.0 {
                    status.mark_invalid(InvalidKind::SqrtNegative);
                    core::f64::NAN
                } else {
                    rsqrt(x)
                }
            }
            Function::Powr | Function::Pown | Function::Rootn => core::f64::NAN,
        }
    }

    pub fn apply_binary(self, a: f64, b: f64, math_mode: MathMode, status: &mut CalcStatus) -> f64 {
        if !matches!(math_mode, MathMode::Ieee754_2019) {
            status.mark_invalid(InvalidKind::UnsupportedInMode);
            return core::f64::NAN;
        }
        match self {
            Function::Powr => powr(a, b, status),
            Function::Pown => pown(a, b, status),
            Function::Rootn => rootn(a, b, status),
            _ => core::f64::NAN,
        }
    }
}

// ==================================================================
// 5. ТОКЕНЫ
// ==================================================================
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Token {
    Number(f64),
    Op(Operator),
    Func(Function),
    LParen,
    RParen,
    Comma,
    UnaryMinus,
    Factorial,
}

// ==================================================================
// 6. ЛЕКСЕР (без аллокатора, сравнение байтов)
// ==================================================================
struct Lexer<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(expr: &'a [u8]) -> Self {
        Self {
            bytes: expr,
            pos: 0,
        }
    }
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
    fn advance(&mut self) {
        self.pos += 1;
    }
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == b' ' || c == b'\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        while let Some(c) = self.peek() {
            let prev = if self.pos > start {
                self.bytes[self.pos - 1]
            } else {
                0
            };
            if c.is_ascii_digit()
                || c == b'.'
                || c == b'e'
                || c == b'E'
                || ((c == b'+' || c == b'-') && (prev == b'e' || prev == b'E'))
            {
                self.advance();
            } else {
                break;
            }
        }
        core::str::from_utf8(&self.bytes[start..self.pos])
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0)
    }

    fn read_ident(&mut self) -> &'a [u8] {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphabetic() || c >= 0x80 {
                self.advance();
            } else {
                break;
            }
        }
        &self.bytes[start..self.pos]
    }

    // Сравнение без учёта регистра для ASCII (латиница)
    fn eq_ignore_ascii_case(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter().zip(b).all(|(x, y)| x.to_ascii_lowercase() == *y)
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let c = self.peek()?;
        match c {
            b'+' => {
                self.advance();
                Some(Token::Op(Operator::Add))
            }
            b'-' => {
                self.advance();
                Some(Token::Op(Operator::Sub))
            }
            b'*' => {
                self.advance();
                Some(Token::Op(Operator::Mul))
            }
            b'/' => {
                self.advance();
                Some(Token::Op(Operator::Div))
            }
            b'^' => {
                self.advance();
                Some(Token::Op(Operator::Pow))
            }
            b'%' => {
                self.advance();
                Some(Token::Op(Operator::PercentOf))
            }
            b'~' => {
                self.advance();
                Some(Token::Op(Operator::PercentTo))
            }
            b'(' => {
                self.advance();
                Some(Token::LParen)
            }
            b')' => {
                self.advance();
                Some(Token::RParen)
            }
            b',' => {
                self.advance();
                Some(Token::Comma)
            }
            b'!' => {
                self.advance();
                Some(Token::Factorial)
            }
            b'0'..=b'9' | b'.' => Some(Token::Number(self.read_number())),
            _ => {
                let ident = self.read_ident();
                if ident.is_empty() {
                    self.advance();
                    return self.next_token();
                }
                // Прямое сравнение байтов (без аллокации)
                if Self::eq_ignore_ascii_case(ident, b"sin") || ident == "синус".as_bytes() {
                    Some(Token::Func(Function::Sin))
                } else if Self::eq_ignore_ascii_case(ident, b"sinpi") {
                    Some(Token::Func(Function::SinPi))
                } else if Self::eq_ignore_ascii_case(ident, b"cos") || ident == "косинус".as_bytes()
                {
                    Some(Token::Func(Function::Cos))
                } else if Self::eq_ignore_ascii_case(ident, b"cospi") {
                    Some(Token::Func(Function::CosPi))
                } else if Self::eq_ignore_ascii_case(ident, b"tan") || ident == "тангенс".as_bytes()
                {
                    Some(Token::Func(Function::Tan))
                } else if Self::eq_ignore_ascii_case(ident, b"tanpi") {
                    Some(Token::Func(Function::TanPi))
                } else if Self::eq_ignore_ascii_case(ident, b"asin")
                    || ident == "арксинус".as_bytes()
                    || ident == "арксин".as_bytes()
                {
                    Some(Token::Func(Function::Asin))
                } else if Self::eq_ignore_ascii_case(ident, b"asinpi") {
                    Some(Token::Func(Function::AsinPi))
                } else if Self::eq_ignore_ascii_case(ident, b"acos")
                    || ident == "арккосинус".as_bytes()
                    || ident == "арккос".as_bytes()
                {
                    Some(Token::Func(Function::Acos))
                } else if Self::eq_ignore_ascii_case(ident, b"acospi") {
                    Some(Token::Func(Function::AcosPi))
                } else if Self::eq_ignore_ascii_case(ident, b"atan")
                    || ident == "арктангенс".as_bytes()
                    || ident == "арктанг".as_bytes()
                {
                    Some(Token::Func(Function::Atan))
                } else if Self::eq_ignore_ascii_case(ident, b"atanpi") {
                    Some(Token::Func(Function::AtanPi))
                } else if Self::eq_ignore_ascii_case(ident, b"sqrt") || ident == "корень".as_bytes()
                {
                    Some(Token::Func(Function::Sqrt))
                } else if Self::eq_ignore_ascii_case(ident, b"rsqrt") {
                    Some(Token::Func(Function::Rsqrt))
                } else if Self::eq_ignore_ascii_case(ident, b"ln") || ident == "лн".as_bytes() {
                    Some(Token::Func(Function::Ln))
                } else if Self::eq_ignore_ascii_case(ident, b"log")
                    || ident == "логарифм".as_bytes()
                    || ident == "лог".as_bytes()
                {
                    Some(Token::Func(Function::Log))
                } else if Self::eq_ignore_ascii_case(ident, b"exp")
                    || ident == "экспонента".as_bytes()
                    || ident == "эксп".as_bytes()
                {
                    Some(Token::Func(Function::Exp))
                } else if Self::eq_ignore_ascii_case(ident, b"exp2m1") {
                    Some(Token::Func(Function::Exp2m1))
                } else if Self::eq_ignore_ascii_case(ident, b"log2p1") {
                    Some(Token::Func(Function::Log2p1))
                } else if Self::eq_ignore_ascii_case(ident, b"log10p1") {
                    Some(Token::Func(Function::Log10p1))
                } else if Self::eq_ignore_ascii_case(ident, b"powr") {
                    Some(Token::Func(Function::Powr))
                } else if Self::eq_ignore_ascii_case(ident, b"pown") {
                    Some(Token::Func(Function::Pown))
                } else if Self::eq_ignore_ascii_case(ident, b"rootn") {
                    Some(Token::Func(Function::Rootn))
                } else if Self::eq_ignore_ascii_case(ident, b"pi") || ident == "пи".as_bytes() {
                    Some(Token::Number(core::f64::consts::PI))
                } else if Self::eq_ignore_ascii_case(ident, b"inf")
                    || ident == "бесконечность".as_bytes()
                {
                    Some(Token::Number(core::f64::INFINITY))
                } else {
                    // Неизвестный идентификатор – считаем умножением (например, "2x" → 2*x)
                    self.pos -= ident.len();
                    Some(Token::Op(Operator::Mul))
                }
            }
        }
    }
}

// ==================================================================
// 7. SHUNTING-YARD (алгоритм сортировочной станции)
// ==================================================================
const TOKEN_STACK_SIZE: usize = 128;

struct RpnBuilder {
    out: [Token; TOKEN_STACK_SIZE],
    out_len: usize,
    ops: [Token; TOKEN_STACK_SIZE],
    ops_len: usize,
}

impl RpnBuilder {
    fn op_info(token: Token) -> Option<(u8, bool)> {
        match token {
            Token::Op(op) => Some((op.precedence(), op.right_assoc())),
            Token::UnaryMinus => Some((4, true)),
            _ => None,
        }
    }

    fn new() -> Self {
        Self {
            out: [Token::Number(0.0); TOKEN_STACK_SIZE],
            out_len: 0,
            ops: [Token::Number(0.0); TOKEN_STACK_SIZE],
            ops_len: 0,
        }
    }

    fn push_out(&mut self, t: Token) {
        if self.out_len < TOKEN_STACK_SIZE {
            self.out[self.out_len] = t;
            self.out_len += 1;
        }
    }
    fn push_op(&mut self, t: Token) {
        if self.ops_len < TOKEN_STACK_SIZE {
            self.ops[self.ops_len] = t;
            self.ops_len += 1;
        }
    }
    fn pop_op(&mut self) -> Option<Token> {
        if self.ops_len > 0 {
            self.ops_len -= 1;
            Some(self.ops[self.ops_len])
        } else {
            None
        }
    }
    fn top_op(&self) -> Option<Token> {
        if self.ops_len > 0 {
            Some(self.ops[self.ops_len - 1])
        } else {
            None
        }
    }

    fn build(&mut self, tokens: &[Token]) {
        let mut expect_unary = true;
        for &tok in tokens {
            match tok {
                Token::Number(_) => {
                    self.push_out(tok);
                    expect_unary = false;
                }
                Token::Func(_) => {
                    self.push_op(tok);
                    expect_unary = true;
                }
                Token::Op(op) => {
                    if expect_unary && op == Operator::Sub {
                        self.push_op(Token::UnaryMinus);
                    } else {
                        while let Some(top) = self.top_op() {
                            if let Some((top_prec, _top_right_assoc)) = Self::op_info(top) {
                                if op.precedence() < top_prec
                                    || (op.precedence() == top_prec && !op.right_assoc())
                                {
                                    if let Some(popped) = self.pop_op() {
                                        self.push_out(popped);
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        self.push_op(Token::Op(op));
                    }
                    expect_unary = true;
                }
                Token::LParen => {
                    self.push_op(Token::LParen);
                    expect_unary = true;
                }
                Token::Comma => {
                    while let Some(top) = self.top_op() {
                        if matches!(top, Token::LParen) {
                            break;
                        }
                        if let Some(popped) = self.pop_op() {
                            self.push_out(popped);
                        }
                    }
                    expect_unary = true;
                }
                Token::RParen => {
                    while let Some(top) = self.pop_op() {
                        if matches!(top, Token::LParen) {
                            break;
                        }
                        self.push_out(top);
                    }
                    if let Some(Token::Func(_)) = self.top_op() {
                        if let Some(f) = self.pop_op() {
                            self.push_out(f);
                        }
                    }
                    expect_unary = false;
                }
                Token::Factorial => {
                    self.push_out(Token::Factorial);
                    expect_unary = false;
                }
                Token::UnaryMinus => {}
            }
        }
        while let Some(op) = self.pop_op() {
            if !matches!(op, Token::LParen) {
                self.push_out(op);
            }
        }
    }
}

// ==================================================================
// 8. ВЫЧИСЛИТЕЛЬ RPN
// ==================================================================
struct RpnEvaluator {
    stack: [f64; TOKEN_STACK_SIZE],
    sp: usize,
}

impl RpnEvaluator {
    fn new() -> Self {
        Self {
            stack: [0.0; TOKEN_STACK_SIZE],
            sp: 0,
        }
    }
    fn push(&mut self, v: f64) {
        if self.sp < TOKEN_STACK_SIZE {
            self.stack[self.sp] = v;
            self.sp += 1;
        }
    }
    fn pop(&mut self) -> f64 {
        if self.sp > 0 {
            self.sp -= 1;
            self.stack[self.sp]
        } else {
            core::f64::NAN
        }
    }

    fn eval(
        &mut self,
        rpn: &[Token],
        angle_mode: AngleMode,
        math_mode: MathMode,
        status: &mut CalcStatus,
    ) -> f64 {
        for &tok in rpn {
            match tok {
                Token::Number(x) => self.push(x),
                Token::Op(op) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(op.apply(a, b, math_mode, status));
                }
                Token::Func(f) => {
                    if f.arity() == 1 {
                        let a = self.pop();
                        self.push(f.apply(a, angle_mode, math_mode, status));
                    } else {
                        let b = self.pop();
                        let a = self.pop();
                        self.push(f.apply_binary(a, b, math_mode, status));
                    }
                }
                Token::UnaryMinus => {
                    let a = self.pop();
                    self.push(-a);
                }
                Token::Factorial => {
                    let a = self.pop();
                    self.push(unsafe { x87::factorial(a) });
                }
                _ => {}
            }
        }
        self.pop()
    }
}

pub fn evaluate_expression(
    expr: &str,
    angle_mode: AngleMode,
    math_mode: MathMode,
) -> (f64, CalcStatus) {
    let mut lex = Lexer::new(expr.as_bytes());
    let mut tokens = [Token::Number(0.0); TOKEN_STACK_SIZE];
    let mut tlen = 0;
    while let Some(token) = lex.next_token() {
        if tlen < TOKEN_STACK_SIZE {
            tokens[tlen] = token;
            tlen += 1;
        }
    }

    let mut builder = RpnBuilder::new();
    builder.build(&tokens[..tlen]);
    let mut evaluator = RpnEvaluator::new();
    let mut status = CalcStatus::new();
    let value = evaluator.eval(
        &builder.out[..builder.out_len],
        angle_mode,
        math_mode,
        &mut status,
    );
    (value, status)
}

// ==================================================================
// 9. ВЫВОД ЧИСЕЛ (без format!, ручное преобразование)
// ==================================================================
fn f64_to_str(val: f64, buf: &mut [u8; 32]) -> &str {
    if val.is_nan() {
        buf[..3].copy_from_slice(b"NaN");
        return core::str::from_utf8(&buf[..3]).unwrap_or("NaN");
    }
    if val.is_infinite() {
        let s: &[u8] = if val.is_sign_positive() {
            b"\xE2\x88\x9E"
        } else {
            b"-\xE2\x88\x9E"
        };
        let len = s.len().min(buf.len());
        buf[..len].copy_from_slice(&s[..len]);
        return core::str::from_utf8(&buf[..len]).unwrap_or("inf");
    }
    if val == 0.0 {
        buf[0] = b'0';
        return core::str::from_utf8(&buf[..1]).unwrap_or("0");
    }

    let neg = val.is_sign_negative();
    let abs = if neg { -val } else { val };
    let mut idx = 0;
    if neg {
        buf[idx] = b'-';
        idx += 1;
    }

    if abs > 1e14 || abs < 1e-6 {
        let mut m = abs;
        let mut exp = 0i32;
        if m >= 10.0 {
            while m >= 10.0 && exp < 400 {
                m /= 10.0;
                exp += 1;
            }
        } else {
            while m < 1.0 && m > 0.0 && exp > -400 {
                m *= 10.0;
                exp -= 1;
            }
        }
        let whole = m as u8;
        buf[idx] = b'0' + (whole % 10);
        idx += 1;
        buf[idx] = b'.';
        idx += 1;
        let mut frac = m - whole as f64;
        for _ in 0..6 {
            if idx >= 30 {
                break;
            }
            frac *= 10.0;
            buf[idx] = b'0' + (frac as u8 % 10);
            idx += 1;
            frac -= trunc_f64(frac);
        }
        buf[idx] = b'e';
        idx += 1;
        buf[idx] = if exp >= 0 { b'+' } else { b'-' };
        idx += 1;
        let exp_abs = exp.unsigned_abs();
        if exp_abs >= 100 {
            buf[idx] = b'0' + (exp_abs / 100) as u8;
            idx += 1;
        }
        buf[idx] = b'0' + ((exp_abs / 10) % 10) as u8;
        idx += 1;
        buf[idx] = b'0' + (exp_abs % 10) as u8;
        idx += 1;
    } else {
        let whole = trunc_f64(abs) as i64;
        if whole == 0 {
            buf[idx] = b'0';
            idx += 1;
        } else {
            let mut tmp = whole;
            let mut digits = [0u8; 20];
            let mut dcnt = 0;
            while tmp > 0 {
                digits[dcnt] = b'0' + (tmp % 10) as u8;
                tmp /= 10;
                dcnt += 1;
            }
            for d in (0..dcnt).rev() {
                buf[idx] = digits[d];
                idx += 1;
            }
        }
        let frac = abs - trunc_f64(abs);
        if frac > 1e-12 {
            buf[idx] = b'.';
            idx += 1;
            let mut f = frac;
            for _ in 0..10 {
                if idx >= 31 {
                    break;
                }
                f *= 10.0;
                buf[idx] = b'0' + ((f as u64) % 10) as u8;
                idx += 1;
                f -= trunc_f64(f);
                if f < 1e-12 {
                    break;
                }
            }
        }
    }
    core::str::from_utf8(&buf[..idx]).unwrap_or("?")
}

// ==================================================================
// 10. СОСТОЯНИЕ И ИНТЕРФЕЙС
// ==================================================================
#[derive(Clone, Copy, PartialEq)]
pub enum AngleMode {
    Rad,
    Deg,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Lang {
    Rus,
    Eng,
}

struct Memory {
    val: f64,
}
impl Memory {
    const fn new() -> Self {
        Self { val: 0.0 }
    }
    fn recall(&self) -> f64 {
        self.val
    }
    fn add(&mut self, x: f64) {
        self.val += x;
    }
    fn clear(&mut self) {
        self.val = 0.0;
    }
}

const BUTTONS_EN: &[&str] = &[
    "sin", "cos", "tan", "sqrt", "C", "asin", "acos", "atan", "^", "/", " 7 ", " 8 ", " 9 ", "*",
    "exp", " 4 ", " 5 ", " 6 ", "-", "log", " 1 ", " 2 ", " 3 ", "+", "ln ", "+/-", " 0 ", " . ",
    "=", "pi ", "INF", " ( ", " ) ", "M+", "MR", "MC", "D/R", "STD", "RUS", "% of", "to%",
];
const BUTTONS_RU: &[&str] = &[
    "синус",
    "косинус",
    "тангенс",
    "корень",
    "C",
    "арксин",
    "арккос",
    "арктанг",
    "^",
    "/",
    " 7 ",
    " 8 ",
    " 9 ",
    "*",
    "эксп",
    " 4 ",
    " 5 ",
    " 6 ",
    "-",
    "лог",
    " 1 ",
    " 2 ",
    " 3 ",
    "+",
    "лн ",
    "+/-",
    " 0 ",
    " . ",
    "=",
    "π ",
    "∞",
    " ( ",
    " ) ",
    "M+",
    "MR",
    "MC",
    "ГРД",
    "СТД",
    "ENG",
    "% от",
    "в %",
];
const COLS: usize = 5;

pub struct CalcApp {
    selected: usize,
    expr_buf: [u8; 256],
    expr_len: usize,
    result: f64,
    show: bool,
    status: CalcStatus,
    angle: AngleMode,
    lang: Lang,
    mem: Memory,
    math_mode: MathMode,
}

impl CalcApp {
    pub const fn new() -> Self {
        Self {
            selected: 0,
            expr_buf: [0; 256],
            expr_len: 0,
            result: 0.0,
            show: false,
            status: CalcStatus::new(),
            angle: AngleMode::Rad,
            lang: Lang::Eng,
            mem: Memory::new(),
            math_mode: MathMode::Ieee754_2019,
        }
    }

    unsafe fn draw(&self) {
        crate::fb_buffer::clear_screen();
        let title = if self.lang == Lang::Eng {
            "--- ENGINEERING CALCULATOR [IEEE 754 / ISO 60559] (x87 80-bit) ---"
        } else {
            "--- ИНЖЕНЕРНЫЙ КАЛЬКУЛЯТОР [IEEE 754 / ISO 60559] (x87 80-bit) ---"
        };
        locale::write_str_at_vga(title, 1, 10, 0x0E);

        // Строка состояния – без format!
        let mode = match (self.angle, self.lang) {
            (AngleMode::Rad, Lang::Eng) => "RAD",
            (AngleMode::Rad, Lang::Rus) => "РАД",
            (AngleMode::Deg, Lang::Eng) => "DEG",
            (AngleMode::Deg, Lang::Rus) => "ГРД",
        };
        locale::write_str_at_vga(mode, 2, 30, 0x0C);
        let std_str = match self.math_mode {
            MathMode::Ieee754_2008 => "08",
            MathMode::Ieee754_2019 => "19",
        };
        locale::write_str_at_vga(std_str, 2, 24, 0x0C);
        let lang_str = if self.lang == Lang::Eng {
            "ENG"
        } else {
            "РУС"
        };
        locale::write_str_at_vga(lang_str, 2, 35, 0x0C);
        if self.mem.val != 0.0 {
            locale::write_str_at_vga(" M", 2, 39, 0x0C);
        }

        locale::write_str_at_vga(" _________________________________________ ", 4, 17, 0x07);
        locale::write_str_at_vga("|                                         |", 5, 17, 0x07);

        // Формирование дисплея с ручным выравниванием (учёт многобайтовых символов)
        let mut dbuf = [0u8; 32];
        let disp_str = if self.show {
            if self.result.is_nan() {
                if let Some(kind) = self.status.invalid {
                    status_tag(kind)
                } else {
                    f64_to_str(self.result, &mut dbuf)
                }
            } else {
                f64_to_str(self.result, &mut dbuf)
            }
        } else {
            core::str::from_utf8(&self.expr_buf[..self.expr_len]).unwrap_or("")
        };
        let chars_count = disp_str.chars().count();
        let extra = if self.show && self.status.indeterminate {
            1
        } else {
            0
        };
        let pad = 19 + (40 - chars_count.min(40) - extra);
        locale::write_str_at_vga(disp_str, 5, pad, 0x0F);
        if self.show && self.status.indeterminate {
            locale::write_str_at_vga("?", 5, pad + chars_count, 0x0F);
        }
        locale::write_str_at_vga("|_________________________________________|", 6, 17, 0x07);

        // Кнопки
        let btns = if self.lang == Lang::Eng {
            BUTTONS_EN
        } else {
            BUTTONS_RU
        };
        for (i, &lbl) in btns.iter().enumerate() {
            let r = i / COLS;
            let c = i % COLS;
            let x = 18 + c * 9;
            let y = 8 + r * 2;
            let col = if i == self.selected { 0x4F } else { 0x07 };
            let label_width = lbl.chars().count();
            locale::write_str_at_vga("[", y, x, 0x08);
            locale::write_str_at_vga(lbl, y, x + 1, col);
            locale::write_str_at_vga("]", y, x + 1 + label_width, 0x08);
        }
        locale::write_str_at_vga(
            "Arrows/Enter: buttons | Keyboard: digits +-*/^%~()! | Backspace | Q: quit",
            24,
            10,
            0x08,
        );
    }

    fn add_char(&mut self, c: u8) {
        if self.show {
            self.expr_len = 0;
            self.show = false;
            self.status.clear();
        }
        if self.expr_len + 1 < self.expr_buf.len() {
            self.expr_buf[self.expr_len] = c;
            self.expr_len += 1;
        }
    }
    fn add_str(&mut self, s: &str) {
        if self.show {
            self.expr_len = 0;
            self.show = false;
            self.status.clear();
        }
        for &b in s.as_bytes() {
            if self.expr_len + 1 < self.expr_buf.len() {
                self.expr_buf[self.expr_len] = b;
                self.expr_len += 1;
            }
        }
    }
    fn backspace(&mut self) {
        if self.show {
            self.show = false;
            return;
        }
        if self.expr_len > 0 {
            self.expr_len -= 1;
            self.expr_buf[self.expr_len] = 0;
        }
    }
    fn clear(&mut self) {
        self.expr_len = 0;
        self.expr_buf = [0; 256];
        self.show = false;
        self.status.clear();
    }

    fn evaluate(&mut self) {
        if self.expr_len == 0 {
            return;
        }
        let expr = core::str::from_utf8(&self.expr_buf[..self.expr_len]).unwrap_or("");
        let (res, status) = evaluate_expression(expr, self.angle, self.math_mode);
        self.result = res;
        self.status = status;
        self.show = true;
    }

    fn handle(&mut self, cmd: &str) {
        match cmd {
            "=" => self.evaluate(),
            "C" => self.clear(),
            "+/-" => {
                if self.show {
                    self.result = -self.result;
                    return;
                }
                if self.expr_len == 0 {
                    return;
                }
                let mut i = self.expr_len;
                while i > 0
                    && (self.expr_buf[i - 1].is_ascii_digit() || self.expr_buf[i - 1] == b'.')
                {
                    i -= 1;
                }
                if i > 0 && self.expr_buf[i - 1] == b'-' {
                    for j in i - 1..self.expr_len - 1 {
                        self.expr_buf[j] = self.expr_buf[j + 1];
                    }
                    self.expr_len -= 1;
                } else {
                    if self.expr_len + 1 < self.expr_buf.len() {
                        for j in (i..self.expr_len).rev() {
                            self.expr_buf[j + 1] = self.expr_buf[j];
                        }
                        self.expr_buf[i] = b'-';
                        self.expr_len += 1;
                    }
                }
            }
            "INF" | "∞" => self.add_str("inf"),
            "pi " | "π " => self.add_str("pi"),
            "n! " | "факт" => self.add_char(b'!'),
            "% of" | "% от" => self.add_char(b'%'),
            "to%" | "в %" => self.add_char(b'~'),
            "sin" | "синус" => self.add_str("sin("),
            "cos" | "косинус" => self.add_str("cos("),
            "tan" | "тангенс" => self.add_str("tan("),
            "asin" | "арксин" => self.add_str("asin("),
            "acos" | "арккос" => self.add_str("acos("),
            "atan" | "арктанг" => self.add_str("atan("),
            "sqrt" | "корень" => self.add_str("sqrt("),
            "exp" | "эксп" => self.add_str("exp("),
            "exp2m1" => self.add_str("exp2m1("),
            "log" | "лог" => self.add_str("log("),
            "log2p1" => self.add_str("log2p1("),
            "log10p1" => self.add_str("log10p1("),
            "powr" => self.add_str("powr("),
            "pown" => self.add_str("pown("),
            "rootn" => self.add_str("rootn("),
            "ln " | "лн " => self.add_str("ln("),
            "M+" => {
                let v = if self.show {
                    self.result
                } else {
                    core::str::from_utf8(&self.expr_buf[..self.expr_len])
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0.0)
                };
                self.mem.add(v);
            }
            "MR" => {
                self.result = self.mem.recall();
                self.status.clear();
                self.show = true;
            }
            "MC" => self.mem.clear(),
            "D/R" | "ГРД" => {
                self.angle = if let AngleMode::Rad = self.angle {
                    AngleMode::Deg
                } else {
                    AngleMode::Rad
                }
            }
            "STD" | "СТД" => {
                self.math_mode = match self.math_mode {
                    MathMode::Ieee754_2008 => MathMode::Ieee754_2019,
                    MathMode::Ieee754_2019 => MathMode::Ieee754_2008,
                };
            }
            "RUS" | "ENG" => {
                self.lang = if let Lang::Eng = self.lang {
                    Lang::Rus
                } else {
                    Lang::Eng
                }
            }
            _ => {
                if self.show {
                    self.expr_len = 0;
                    self.show = false;
                    self.status.clear();
                }
                for &b in cmd.as_bytes() {
                    if self.expr_len + 1 < self.expr_buf.len() {
                        self.expr_buf[self.expr_len] = b;
                        self.expr_len += 1;
                    }
                }
            }
        }
    }

    pub unsafe fn run(&mut self) {
        self.draw();
        loop {
            let mut dirty = false;
            match crate::apps::games::common_hw::poll_input() {
                GameInput::Up => {
                    if self.selected >= COLS {
                        self.selected -= COLS;
                        dirty = true;
                    }
                }
                GameInput::Down => {
                    let lim = if self.lang == Lang::Eng {
                        BUTTONS_EN.len()
                    } else {
                        BUTTONS_RU.len()
                    };
                    if self.selected + COLS < lim {
                        self.selected += COLS;
                        dirty = true;
                    }
                }
                GameInput::Left => {
                    if self.selected > 0 {
                        self.selected -= 1;
                        dirty = true;
                    }
                }
                GameInput::Right => {
                    let lim = if self.lang == Lang::Eng {
                        BUTTONS_EN.len()
                    } else {
                        BUTTONS_RU.len()
                    };
                    if self.selected + 1 < lim {
                        self.selected += 1;
                        dirty = true;
                    }
                }
                GameInput::Char(b'0') => {
                    self.add_char(b'0');
                    dirty = true;
                }
                GameInput::Char(b'1') => {
                    self.add_char(b'1');
                    dirty = true;
                }
                GameInput::Char(b'2') => {
                    self.add_char(b'2');
                    dirty = true;
                }
                GameInput::Char(b'3') => {
                    self.add_char(b'3');
                    dirty = true;
                }
                GameInput::Char(b'4') => {
                    self.add_char(b'4');
                    dirty = true;
                }
                GameInput::Char(b'5') => {
                    self.add_char(b'5');
                    dirty = true;
                }
                GameInput::Char(b'6') => {
                    self.add_char(b'6');
                    dirty = true;
                }
                GameInput::Char(b'7') => {
                    self.add_char(b'7');
                    dirty = true;
                }
                GameInput::Char(b'8') => {
                    self.add_char(b'8');
                    dirty = true;
                }
                GameInput::Char(b'9') => {
                    self.add_char(b'9');
                    dirty = true;
                }
                GameInput::Char(b'.') | GameInput::Char(b',') => {
                    self.add_char(b'.');
                    dirty = true;
                }
                GameInput::Char(b'-') => {
                    self.add_char(b'-');
                    dirty = true;
                }
                GameInput::Char(b'+') => {
                    self.add_char(b'+');
                    dirty = true;
                }
                GameInput::Char(b'*') => {
                    self.add_char(b'*');
                    dirty = true;
                }
                GameInput::Char(b'/') => {
                    self.add_char(b'/');
                    dirty = true;
                }
                GameInput::Char(b'^') => {
                    self.add_char(b'^');
                    dirty = true;
                }
                GameInput::Char(b'%') => {
                    self.add_char(b'%');
                    dirty = true;
                }
                GameInput::Char(b'~') => {
                    self.add_char(b'~');
                    dirty = true;
                }
                GameInput::Char(b'!') => {
                    self.add_char(b'!');
                    dirty = true;
                }
                GameInput::Char(b'(') => {
                    self.add_char(b'(');
                    dirty = true;
                }
                GameInput::Char(b')') => {
                    self.add_char(b')');
                    dirty = true;
                }
                GameInput::Char(b'p') | GameInput::Char(b'P') => {
                    self.add_str("pi");
                    dirty = true;
                }
                GameInput::Char(b'i') | GameInput::Char(b'I') => {
                    self.add_str("inf");
                    dirty = true;
                }
                GameInput::Char(8) => {
                    self.backspace();
                    dirty = true;
                }
                GameInput::Confirm => {
                    let btns = if self.lang == Lang::Eng {
                        BUTTONS_EN
                    } else {
                        BUTTONS_RU
                    };
                    self.handle(btns[self.selected].trim());
                    dirty = true;
                }
                GameInput::Back => break,
                _ => {}
            }
            if dirty {
                self.draw();
            }
        }
    }
}

pub fn run() {
    let mut app = CalcApp::new();
    unsafe {
        app.run();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_expr(expr: &str, math_mode: MathMode) -> (f64, CalcStatus) {
        evaluate_expression(expr, AngleMode::Rad, math_mode)
    }

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-9
    }

    #[test_case]
    fn smoke_2008_zero_pow_zero_is_nan() {
        let (value, status) = eval_expr("0^0", MathMode::Ieee754_2008);
        assert!(value.is_nan());
        assert_eq!(status.invalid, Some(InvalidKind::PowZeroZero));
        assert!(status.indeterminate);
    }

    #[test_case]
    fn smoke_2019_zero_pow_zero_is_finite_indeterminate() {
        let (value, status) = eval_expr("0^0", MathMode::Ieee754_2019);
        assert!(approx_eq(value, 1.0));
        assert_eq!(status.invalid, Some(InvalidKind::PowZeroZero));
        assert!(status.indeterminate);
    }

    #[test_case]
    fn smoke_indeterminate_forms_are_tagged() {
        let (zero_div_zero, status_zero_div_zero) = eval_expr("0/0", MathMode::Ieee754_2019);
        assert!(zero_div_zero.is_nan());
        assert_eq!(status_zero_div_zero.invalid, Some(InvalidKind::ZeroDivZero));

        let (inf_minus_inf, status_inf_minus_inf) = eval_expr("inf-inf", MathMode::Ieee754_2019);
        assert!(inf_minus_inf.is_nan());
        assert_eq!(status_inf_minus_inf.invalid, Some(InvalidKind::InfMinusInf));

        let (zero_times_inf, status_zero_times_inf) = eval_expr("0*inf", MathMode::Ieee754_2019);
        assert!(zero_times_inf.is_nan());
        assert_eq!(
            status_zero_times_inf.invalid,
            Some(InvalidKind::ZeroTimesInf)
        );
    }

    #[test_case]
    fn smoke_2019_special_functions_work() {
        let (sinpi_half, sinpi_status) = eval_expr("sinpi(0.5)", MathMode::Ieee754_2019);
        assert!(approx_eq(sinpi_half, 1.0));
        assert_eq!(sinpi_status.invalid, None);

        let (pown_value, pown_status) = eval_expr("pown(2,3)", MathMode::Ieee754_2019);
        assert!(approx_eq(pown_value, 8.0));
        assert_eq!(pown_status.invalid, None);

        let (rootn_value, rootn_status) = eval_expr("rootn(27,3)", MathMode::Ieee754_2019);
        assert!(approx_eq(rootn_value, 3.0));
        assert_eq!(rootn_status.invalid, None);
    }

    #[test_case]
    fn smoke_2019_invalid_binary_functions_are_tagged() {
        let (powr_value, powr_status) = eval_expr("powr(-2,0.5)", MathMode::Ieee754_2019);
        assert!(powr_value.is_nan());
        assert_eq!(powr_status.invalid, Some(InvalidKind::NegBaseFracPow));

        let (pown_value, pown_status) = eval_expr("pown(2,0.5)", MathMode::Ieee754_2019);
        assert!(pown_value.is_nan());
        assert_eq!(pown_status.invalid, Some(InvalidKind::NonIntegerArg));

        let (rootn_value, rootn_status) = eval_expr("rootn(9,0)", MathMode::Ieee754_2019);
        assert!(rootn_value.is_nan());
        assert_eq!(rootn_status.invalid, Some(InvalidKind::RootZero));
    }
}
