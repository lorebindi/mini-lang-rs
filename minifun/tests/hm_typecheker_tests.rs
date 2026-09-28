use minifun::ast::{Term, BinOp};
use minifun::typing::types::{Type, TypeSchema};
use minifun::typing::errors::{TypecheckError, TypeError, AnnotationError, OperandPosition};
use minifun::typing::HmTypeChecker;
use minifun::typing::Typechecker;

mod common;
use common::*;

fn tc(t: &Term) -> Result<Type, TypecheckError> {
    HmTypeChecker::new().typecheck(t)
}

// Literals

#[test]
fn num_literal_is_int() {
    assert_eq!(tc(&num(42)), Ok(t_int()));
}

#[test]
fn true_is_bool() {
    assert_eq!(tc(&Term::True), Ok(t_bool()));
}

#[test]
fn false_is_bool() {
    assert_eq!(tc(&Term::False), Ok(t_bool()));
}

// BinOp

#[test]
fn add_of_ints_is_int() {
    assert_eq!(tc(&bin(BinOp::Add, num(1), num(2))), Ok(Type::Int));
}

#[test]
fn lt_of_ints_is_bool() {
    assert_eq!(tc(&bin(BinOp::Lt, num(1), num(2))), Ok(Type::Bool));
}

#[test]
fn and_of_bools_is_bool() {
    assert_eq!(tc(&bin(BinOp::And, Term::True, Term::False)), Ok(t_bool()));
}

#[test]
fn add_left_operand_mismatch() {
    let result = tc(&bin(BinOp::Add, Term::True, num(1)));
    assert_eq!(
        result,
        Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {
            op: BinOp::Add,
            position: OperandPosition::Left,
            expected: t_int(),
            actual: t_bool(),
        }))
    );
}

#[test]
fn add_right_operand_mismatch() {
    let result = tc(&bin(BinOp::Add, num(1), Term::True));
    assert_eq!(
        result,
        Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {
            op: BinOp::Add,
            position: OperandPosition::Right,
            expected: Type::Int,
            actual: Type::Bool,
        }))
    );
}

#[test]
fn and_operand_mismatch() {
    let result = tc(&bin(BinOp::And, num(1), Term::True));
    assert_eq!(
        result,
        Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {
            op: BinOp::And,
            position: OperandPosition::Left,
            expected: Type::Bool,
            actual: Type::Int,
        }))
    );
}

// Not

#[test]
fn not_of_bool_is_bool() {
    assert_eq!(tc(&not_(Term::True)), Ok(Type::Bool));
}

#[test]
fn not_of_int_is_error() {
    assert_eq!(
        tc(&not_(num(1))),
        Err(TypecheckError::Type(TypeError::InvalidNotOperand { actual: Type::Int }))
    );
}

// If

#[test]
fn if_with_matching_branches() {
    assert_eq!(tc(&iff(Term::True, num(1), num(2))), Ok(Type::Int));
}

#[test]
fn if_condition_not_bool() {
    assert_eq!(
        tc(&iff(num(1), num(1), num(2))),
        Err(TypecheckError::Type(TypeError::IfConditionMismatch { actual: Type::Int }))
    );
}

#[test]
fn if_branch_mismatch() {
    assert_eq!(
        tc(&iff(Term::True, num(1), Term::True)),
        Err(TypecheckError::Type(TypeError::BranchMismatch {
            then_type: Type::Int,
            else_type: Type::Bool,
        }))
    );
}

// Var

#[test]
fn undefined_variable() {
    assert_eq!(
        tc(&var("x")),
        Err(TypecheckError::Type(TypeError::UndefinedVariable("x".to_string())))
    );
}

// Fun / App

#[test]
fn identity_function_type() {
    let result = tc(&fun_unannotated("x", var("x")));
    match result {
        Ok(Type::Arrow(param, ret)) => assert_eq!(param, ret),
        other => panic!("expected Arrow(a, a), got {other:?}"),
    }
}

#[test]
fn fun_with_annotation_is_rejected() {
    let result = tc(&fun("x", t_int(), var("x")));
    assert!(matches!(
    result,
    Err(TypecheckError::Annotation(AnnotationError::Unexpected { .. }))
    ));
}

#[test]
fn app_basic() {
    let f = fun_unannotated("x", bin(BinOp::Add, var("x"), num(1)));
    assert_eq!(tc(&app(f, num(5))), Ok(t_int()));
}

#[test]
fn app_on_non_function() {
    assert_eq!(
        tc(&app(num(5), num(3))),
        Err(TypecheckError::Type(TypeError::NotAFunction { actual: Type::Int }))
    );
}

#[test]
fn app_argument_type_mismatch() {
    let f = fun_unannotated("x", bin(BinOp::Add, var("x"), num(1)));
    assert_eq!(
        tc(&app(f, Term::True)),
        Err(TypecheckError::Type(TypeError::ArgTypeMismatch {
            expected: t_int(),
            actual: t_bool(),
        }))
    );
}

// Let e let-polymorphism

#[test]
fn let_simple() {
    assert_eq!(
        tc(&let_("x", num(5), bin(BinOp::Add, var("x"), num(1)))),
        Ok(Type::Int)
    );
}

#[test]
fn let_polymorphism_same_binding_two_types() {
    let id_def = fun_unannotated("x", var("x"));
    let body = iff(
        app(var("id"), Term::True),
        app(var("id"), num(1)),
        app(var("id"), num(2)),
    );
    assert_eq!(tc(&let_("id", id_def, body)), Ok(t_int()));
}

#[test]
fn let_does_not_generalize_context_vars() {
    // fun x => let y = x in if (not y) then (y + 1) else 0   -> error
    let body = let_("y", var("x"),
                    iff(not_(var("y")), bin(BinOp::Add, var("y"), num(1)), num(0)));
    assert!(tc(&fun_unannotated("x", body)).is_err());
}

#[test]
fn lambda_bound_parameter_is_not_generalized() {
    // fun f => if (f true) then (f 1) else (f 2)
    // 'f' it's a lambada parameter: monomorphic.
    // Using it for Bool -> ? and then for Int -> ? in the same body, must fail.
    let body = iff(
        app(var("f"), Term::True),
        app(var("f"), num(1)),
        app(var("f"), num(2)),
    );
    let result = tc(&fun_unannotated("f", body));
    assert!(result.is_err(), "expected a type error, got {result:?}");
}

// LetFun

#[test]
fn letfun_factorial() {
    let body = iff(
        bin(BinOp::Lt, var("n"), num(1)),
        num(1),
        bin(BinOp::Mul, var("n"), app(var("fact"), bin(BinOp::Sub, var("n"), num(1)))),
    );
    let term = letfun_unannotated("fact", "n", body, app(var("fact"), num(5)));
    assert_eq!(tc(&term), Ok(t_int()));
}

#[test]
fn letfun_with_annotation_is_rejected() {
    let term = letfun("f", "x", t_int(), num(1), var("f"));
    assert!(matches!(
        tc(&term),
        Err(TypecheckError::Annotation(AnnotationError::Unexpected { .. }))
    ));
}

#[test]
fn letfun_is_generalized_for_in_term() {
    let body = var("x");
    let in_term = iff(app(var("f"), Term::True), app(var("f"), num(1)), app(var("f"), num(2)));
    let term = letfun_unannotated("f", "x", body, in_term);
    assert_eq!(tc(&term), Ok(t_int()));
}

#[test]
fn letfun_recursion_is_monomorphic_in_body() {
    // letfun f x = if (f true) then (f 1) else 0 in f 1   -> error
    let body = iff(app(var("f"), Term::True), app(var("f"), num(1)), num(0));
    assert!(tc(&letfun_unannotated("f", "x", body, app(var("f"), num(1)))).is_err());
}

#[test]
fn letfun_infinite_type() {
    // letfun f x = f in f
    let t = letfun_unannotated("f", "x", var("f"), var("f"));
    assert!(matches!(tc(&t), Err(TypecheckError::Type(TypeError::InfiniteType { .. }))));
}

#[test]
fn letfun_param_not_visible_in_in_term() {
    let t = letfun_unannotated("f", "x", var("x"), var("x"));
    assert_eq!(
        tc(&t),
        Err(TypecheckError::Type(TypeError::UndefinedVariable("x".to_string())))
    );
}

// Occurs check

#[test]
fn occurs_check_fails_on_self_application() {
    let term = fun_unannotated("x", app(var("x"), var("x")));
    assert!(matches!(
        tc(&term),
        Err(TypecheckError::Type(TypeError::InfiniteType { .. }))
    ));
}



