//! Unit tests for `--font-size` validation.

use super::validate_font_size;
use crate::error::CliError;

#[test]
fn validate_font_size_passes_none_through() {
    let result = validate_font_size(None).expect("none must be accepted");
    assert_eq!(result, None);
}

#[test]
fn validate_font_size_accepts_positive_finite() {
    let result = validate_font_size(Some(12.0)).expect("positive finite must be accepted");
    assert_eq!(result, Some(12.0));
}

#[test]
fn validate_font_size_accepts_small_positive_value() {
    let result =
        validate_font_size(Some(f32::MIN_POSITIVE)).expect("subnormal-adjacent value accepted");
    assert_eq!(result, Some(f32::MIN_POSITIVE));
}

#[test]
fn validate_font_size_rejects_zero() {
    let result = validate_font_size(Some(0.0));
    assert!(matches!(result, Err(CliError::InvalidFontSize(value)) if value == 0.0));
}

#[test]
fn validate_font_size_rejects_negative_zero() {
    let result = validate_font_size(Some(-0.0));
    // `-0.0 <= 0.0` so negative zero is rejected. The carried value compares
    // equal to 0.0 under `==`.
    assert!(matches!(result, Err(CliError::InvalidFontSize(value)) if value == 0.0));
}

#[test]
fn validate_font_size_rejects_negative() {
    let result = validate_font_size(Some(-1.0));
    assert!(matches!(result, Err(CliError::InvalidFontSize(value)) if value == -1.0));
}

#[test]
fn validate_font_size_rejects_nan() {
    let result = validate_font_size(Some(f32::NAN));
    let Err(CliError::InvalidFontSize(value)) = result else {
        panic!("NaN must be rejected; got {result:?}");
    };
    assert!(value.is_nan(), "carried value must be NaN");
}

#[test]
fn validate_font_size_rejects_positive_infinity() {
    let result = validate_font_size(Some(f32::INFINITY));
    assert!(matches!(result, Err(CliError::InvalidFontSize(value)) if value == f32::INFINITY));
}

#[test]
fn validate_font_size_rejects_negative_infinity() {
    let result = validate_font_size(Some(f32::NEG_INFINITY));
    assert!(matches!(result, Err(CliError::InvalidFontSize(value)) if value == f32::NEG_INFINITY));
}
