#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(neroshiza_dev_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use neroshiza_dev_os::apps::calculator::{AngleMode, InvalidKind, MathMode, evaluate_expression};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    neroshiza_dev_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    neroshiza_dev_os::test_panic_handler(info)
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

#[test_case]
fn smoke_2008_zero_pow_zero_is_nan() {
    let (value, status) = evaluate_expression("0^0", AngleMode::Rad, MathMode::Ieee754_2008);
    assert!(value.is_nan());
    assert_eq!(status.invalid, Some(InvalidKind::PowZeroZero));
    assert!(status.indeterminate);
}

#[test_case]
fn smoke_2019_zero_pow_zero_is_finite_indeterminate() {
    let (value, status) = evaluate_expression("0^0", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(approx_eq(value, 1.0));
    assert_eq!(status.invalid, Some(InvalidKind::PowZeroZero));
    assert!(status.indeterminate);
}

#[test_case]
fn smoke_indeterminate_forms_are_tagged() {
    let (zero_div_zero, status_zero_div_zero) =
        evaluate_expression("0/0", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(zero_div_zero.is_nan());
    assert_eq!(status_zero_div_zero.invalid, Some(InvalidKind::ZeroDivZero));

    let (inf_minus_inf, status_inf_minus_inf) =
        evaluate_expression("inf-inf", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(inf_minus_inf.is_nan());
    assert_eq!(status_inf_minus_inf.invalid, Some(InvalidKind::InfMinusInf));

    let (zero_times_inf, status_zero_times_inf) =
        evaluate_expression("0*inf", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(zero_times_inf.is_nan());
    assert_eq!(
        status_zero_times_inf.invalid,
        Some(InvalidKind::ZeroTimesInf)
    );
}

#[test_case]
fn smoke_2019_special_functions_work() {
    let (sinpi_half, sinpi_status) =
        evaluate_expression("sinpi(0.5)", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(approx_eq(sinpi_half, 1.0));
    assert_eq!(sinpi_status.invalid, None);

    let (pown_value, pown_status) =
        evaluate_expression("pown(2,3)", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(approx_eq(pown_value, 8.0));
    assert_eq!(pown_status.invalid, None);

    let (rootn_value, rootn_status) =
        evaluate_expression("rootn(27,3)", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(approx_eq(rootn_value, 3.0));
    assert_eq!(rootn_status.invalid, None);
}

#[test_case]
fn smoke_2019_invalid_binary_functions_are_tagged() {
    let (powr_value, powr_status) =
        evaluate_expression("powr(-2,0.5)", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(powr_value.is_nan());
    assert_eq!(powr_status.invalid, Some(InvalidKind::NegBaseFracPow));

    let (pown_value, pown_status) =
        evaluate_expression("pown(2,0.5)", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(pown_value.is_nan());
    assert_eq!(pown_status.invalid, Some(InvalidKind::NonIntegerArg));

    let (rootn_value, rootn_status) =
        evaluate_expression("rootn(9,0)", AngleMode::Rad, MathMode::Ieee754_2019);
    assert!(rootn_value.is_nan());
    assert_eq!(rootn_status.invalid, Some(InvalidKind::RootZero));
}
