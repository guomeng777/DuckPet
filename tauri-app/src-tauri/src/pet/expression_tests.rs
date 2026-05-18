use std::collections::HashSet;

use super::expression::{evaluate_expression, ExpressionContext, ExpressionError};

fn context() -> ExpressionContext {
    ExpressionContext::new(1920, 1080, 1920, 1040, 64, 48)
        .with_seed(1)
        .with_spawn_random(37)
        .with_image_position(120, 240)
        .with_parent_position(300, 420)
}

#[test]
fn evaluates_constant_integer() {
    let mut context = context();

    assert_eq!(200, evaluate_expression("200", &mut context).unwrap());
}

#[test]
fn evaluates_screen_dimension_expression() {
    let mut context = context();

    assert_eq!(1930, evaluate_expression("screenW+10", &mut context).unwrap());
}

#[test]
fn evaluates_work_area_minus_image_expression() {
    let mut context = context();

    assert_eq!(992, evaluate_expression("areaH-imageH", &mut context).unwrap());
}

#[test]
fn evaluates_random_spawn_expression() {
    let mut context = context();
    let mut values = HashSet::new();

    for _ in 0..10 {
        let value = evaluate_expression("random*(screenW-imageW-50)/100+25", &mut context)
            .expect("random expression should evaluate");
        assert!((25..=1831).contains(&value));
        values.insert(value);
    }

    assert!(values.len() > 1);
}

#[test]
fn keeps_rands_stable_in_context() {
    let mut context = context();

    assert_eq!(74, evaluate_expression("randS+randS", &mut context).unwrap());
    assert_eq!(74, evaluate_expression("randS+randS", &mut context).unwrap());
}

#[test]
fn supports_parent_and_image_position_variables() {
    let mut context = context();

    assert_eq!(360, evaluate_expression("imageX+parentX-60", &mut context).unwrap());
    assert_eq!(660, evaluate_expression("imageY+parentY", &mut context).unwrap());
}

#[test]
fn respects_precedence_parentheses_unary_and_float_truncation() {
    let mut context = context();

    assert_eq!(14, evaluate_expression("2+3*4", &mut context).unwrap());
    assert_eq!(20, evaluate_expression("(2+3)*4", &mut context).unwrap());
    assert_eq!(-13, evaluate_expression("-(10+3.9)", &mut context).unwrap());
}

#[test]
fn rejects_unknown_variables() {
    let mut context = context();

    assert_eq!(
        ExpressionError::UnknownVariable("System".to_owned()),
        evaluate_expression("System.IO.File", &mut context).unwrap_err()
    );
}

#[test]
fn rejects_division_by_zero() {
    let mut context = context();

    assert_eq!(
        ExpressionError::DivideByZero,
        evaluate_expression("10/(5-5)", &mut context).unwrap_err()
    );
}
