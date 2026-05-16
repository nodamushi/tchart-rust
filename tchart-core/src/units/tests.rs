use super::{Length, LengthError, Px};

#[test]
fn px_zero_constant_is_zero() {
    assert_eq!(Px::ZERO, Px(0.0));
}

#[test]
fn px_to_f32_returns_inner_value() {
    assert!((Px(3.5).to_f32() - 3.5).abs() < f32::EPSILON);
    assert!((Px::ZERO.to_f32() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn px_try_from_positive_finite_accepts_positive() {
    assert_eq!(Px::try_from_positive_finite(12.0), Ok(Px(12.0)));
    assert_eq!(
        Px::try_from_positive_finite(f32::MIN_POSITIVE),
        Ok(Px(f32::MIN_POSITIVE))
    );
}

#[test]
fn px_try_from_positive_finite_rejects_zero_and_negative() {
    assert_eq!(Px::try_from_positive_finite(0.0), Err(0.0));
    assert_eq!(Px::try_from_positive_finite(-0.0), Err(-0.0));
    assert_eq!(Px::try_from_positive_finite(-1.0), Err(-1.0));
}

#[test]
fn px_try_from_positive_finite_rejects_non_finite() {
    assert!(Px::try_from_positive_finite(f32::NAN).is_err());
    assert_eq!(
        Px::try_from_positive_finite(f32::INFINITY),
        Err(f32::INFINITY)
    );
    assert_eq!(
        Px::try_from_positive_finite(f32::NEG_INFINITY),
        Err(f32::NEG_INFINITY)
    );
}

#[test]
fn test_px_scalar_division() {
    assert_eq!(Px(6.0) / 2.0, Px(3.0));
}

#[test]
fn test_px_addition() {
    assert_eq!(Px(1.0) + Px(2.0), Px(3.0));
}

#[test]
fn test_px_subtraction() {
    assert_eq!(Px(5.0) - Px(2.0), Px(3.0));
}

#[test]
fn test_px_scalar_multiplication() {
    assert_eq!(Px(2.0) * 3.0, Px(6.0));
}

#[test]
fn px_max_returns_larger() {
    assert_eq!(Px(1.0).max(Px(2.0)), Px(2.0));
    assert_eq!(Px(3.0).max(Px(2.0)), Px(3.0));
}

#[test]
fn test_px_partial_ord() {
    assert!(Px(1.0) < Px(2.0));
    assert!(Px(2.0) > Px(1.0));
    assert!(Px(1.0) <= Px(1.0));
}

#[test]
fn length_resolve_lh_uses_line_height() {
    let value = Length::new_lh(2.5).expect("valid lh");
    assert_eq!(value.resolve(Px(20.0)), Px(50.0));
}

#[test]
fn length_resolve_px_ignores_line_height() {
    let value = Length::new_px(20.0).expect("valid px");
    assert_eq!(value.resolve(Px(99.0)), Px(20.0));
}

#[test]
fn length_new_lh_rejects_negative() {
    assert_eq!(Length::new_lh(-1.0), Err(LengthError::Negative));
}

#[test]
fn length_new_px_rejects_negative() {
    assert_eq!(Length::new_px(-0.5), Err(LengthError::Negative));
}

#[test]
fn length_new_lh_rejects_nan() {
    assert_eq!(Length::new_lh(f32::NAN), Err(LengthError::NotFinite));
}

#[test]
fn length_new_lh_accepts_zero() {
    assert!(Length::new_lh(0.0).is_ok());
    assert!(Length::new_px(0.0).is_ok());
}
