use minifun::ast::*;
use minifun::typechecker::*;
use minifun::parser::parse_term;

mod common;
use common::*;

// helpers
fn empty_env() -> Context {
    Context::new()
}

fn assert_type(term: &Term, expected: Type) {
    let env = empty_env();
    assert_eq!(typecheck(term, &env), Ok(expected));
}

fn assert_type_in(term: &Term, env: &Context, expected: Type) {
    assert_eq!(typecheck(term, env), Ok(expected));
}

fn assert_err(term: &Term, expected: TypeError) {
    let env = empty_env();
    assert_eq!(typecheck(term, &env), Err(expected));
}

fn assert_program_type(input: &str, expected: Type) {
    let term = parse_term(input).expect("Parsing fallito");
    assert_type(&term, expected);
}

fn assert_program_err(input: &str, expected: TypeError) {
    let term = parse_term(input).expect("Parsing fallito");
    assert_err(&term, expected);
}

// ---------- literals ----------

#[test]
fn test_num_is_int() {
    assert_type(&num(42), t_int());
}

#[test]
fn test_negative_num_is_int() {
    assert_type(&num(-5), t_int());
}

#[test]
fn test_true_is_bool() {
    assert_type(&Term::True, t_bool());
}

#[test]
fn test_false_is_bool() {
    assert_type(&Term::False, t_bool());
}

// ---------- variables ----------

#[test]
fn test_var_defined() {
    let env = empty_env().extend("x".to_string(), t_int());
    assert_type_in(&var("x"), &env, t_int());
}

#[test]
fn test_var_undefined() {
    assert_err(&var("x"), TypeError::UndefinedVariable("x".to_string()));
}

#[test]
fn test_var_shadowing_uses_innermost_binding() {
    let env = empty_env()
        .extend("x".to_string(), t_int())
        .extend("x".to_string(), t_bool());
    assert_type_in(&var("x"), &env, t_bool());
}

// ---------- arithmetic binops ----------

#[test]
fn test_add_ints() {
    assert_type(&bin(BinOp::Add, num(1), num(2)), t_int());
}

#[test]
fn test_sub_ints() {
    assert_type(&bin(BinOp::Sub, num(5), num(2)), t_int());
}

#[test]
fn test_mul_ints() {
    assert_type(&bin(BinOp::Mul, num(3), num(4)), t_int());
}

#[test]
fn test_add_left_operand_not_int() {
    assert_err(
        &bin(BinOp::Add, Term::True, num(1)),
        TypeError::BinOpTypeMismatch {
            op: BinOp::Add,
            position: OperandPosition::Left,
            expected: t_int(),
            actual: t_bool(),
        },
    );
}

#[test]
fn test_add_right_operand_not_int() {
    assert_err(
        &bin(BinOp::Add, num(1), Term::False),
        TypeError::BinOpTypeMismatch {
            op: BinOp::Add,
            position: OperandPosition::Right,
            expected: t_int(),
            actual: t_bool(),
        },
    );
}

#[test]
fn test_mul_both_operands_wrong_type_reports_left_first() {
    // left operand should be checked (and fail) before the right one
    assert_err(
        &bin(BinOp::Mul, Term::True, Term::False),
        TypeError::BinOpTypeMismatch {
            op: BinOp::Mul,
            position: OperandPosition::Left,
            expected: t_int(),
            actual: t_bool(),
        },
    );
}

#[test]
fn test_binop_propagates_inner_error() {
    // the ill-typed subterm's error should surface, not a mismatch about it
    assert_err(&bin(BinOp::Add, var("undefined"), num(1)), TypeError::UndefinedVariable("undefined".to_string()));
}

// ---------- comparison ----------

#[test]
fn test_lt_ints_yields_bool() {
    assert_type(&bin(BinOp::Lt, num(1), num(2)), t_bool());
}

#[test]
fn test_lt_left_operand_not_int() {
    assert_err(
        &bin(BinOp::Lt, Term::True, num(1)),
        TypeError::BinOpTypeMismatch {
            op: BinOp::Lt,
            position: OperandPosition::Left,
            expected: t_int(),
            actual: t_bool(),
        },
    );
}

#[test]
fn test_lt_right_operand_not_int() {
    assert_err(
        &bin(BinOp::Lt, num(1), Term::True),
        TypeError::BinOpTypeMismatch {
            op: BinOp::Lt,
            position: OperandPosition::Right,
            expected: t_int(),
            actual: t_bool(),
        },
    );
}

// ---------- logical and ----------

#[test]
fn test_and_bools() {
    assert_type(&bin(BinOp::And, Term::True, Term::False), t_bool());
}

#[test]
fn test_and_left_operand_not_bool() {
    assert_err(
        &bin(BinOp::And, num(1), Term::True),
        TypeError::BinOpTypeMismatch {
            op: BinOp::And,
            position: OperandPosition::Left,
            expected: t_bool(),
            actual: t_int(),
        },
    );
}

#[test]
fn test_and_right_operand_not_bool() {
    assert_err(
        &bin(BinOp::And, Term::True, num(1)),
        TypeError::BinOpTypeMismatch {
            op: BinOp::And,
            position: OperandPosition::Right,
            expected: t_bool(),
            actual: t_int(),
        },
    );
}

// ---------- not ----------

#[test]
fn test_not_bool() {
    assert_type(&not_(Term::True), t_bool());
}

#[test]
fn test_not_not_bool() {
    assert_type(&not_(not_(Term::False)), t_bool());
}

#[test]
fn test_not_non_bool_operand() {
    assert_err(&not_(num(1)), TypeError::InvalidNotOperand { actual: t_int() });
}

// ---------- if ----------

#[test]
fn test_if_both_branches_int() {
    assert_type(&iff(Term::True, num(1), num(0)), t_int());
}

#[test]
fn test_if_both_branches_bool() {
    assert_type(&iff(Term::False, Term::True, Term::False), t_bool());
}

#[test]
fn test_if_condition_not_bool() {
    assert_err(
        &iff(num(1), num(1), num(0)),
        TypeError::IfConditionMismatch { actual: t_int() },
    );
}

#[test]
fn test_if_branch_type_mismatch() {
    assert_err(
        &iff(Term::True, num(1), Term::False),
        TypeError::BranchMismatch { then_type: t_int(), else_type: t_bool() },
    );
}

#[test]
fn test_if_nested_in_then_branch() {
    assert_type(
        &iff(
            Term::True,
            iff(Term::False, num(1), num(2)),
            num(3),
        ),
        t_int(),
    );
}

#[test]
fn test_if_condition_checked_before_branches() {
    // condition error should surface even if branches would also be ill-typed
    assert_err(
        &iff(num(1), var("nope"), var("nope2")),
        TypeError::IfConditionMismatch { actual: t_int() },
    );
}

// ---------- fun / lambda ----------

#[test]
fn test_fun_simple_identity() {
    assert_type(&fun("x", t_int(), var("x")), t_arr(t_int(), t_int()));
}

#[test]
fn test_fun_body_uses_param_type() {
    assert_type(
        &fun("x", t_int(), bin(BinOp::Add, var("x"), num(1))),
        t_arr(t_int(), t_int()),
    );
}

#[test]
fn test_fun_returning_bool() {
    assert_type(
        &fun("x", t_int(), bin(BinOp::Lt, var("x"), num(0))),
        t_arr(t_int(), t_bool()),
    );
}

#[test]
fn test_fun_body_ill_typed() {
    assert_err(
        &fun("x", t_int(), not_(var("x"))),
        TypeError::InvalidNotOperand { actual: t_int() },
    );
}

#[test]
fn test_fun_param_shadows_outer_binding() {
    let env = empty_env().extend("x".to_string(), t_bool());
    // the parameter x: Int should shadow the outer x: Bool inside the body
    assert_type_in(&fun("x", t_int(), var("x")), &env, t_arr(t_int(), t_int()));
}

#[test]
fn test_higher_order_fun_type() {
    // fun f : Int -> Int => fun x : Int => f x
    let term = fun(
        "f",
        t_arr(t_int(), t_int()),
        fun("x", t_int(), app(var("f"), var("x"))),
    );
    assert_type(&term, t_arr(t_arr(t_int(), t_int()), t_arr(t_int(), t_int())));
}

// ---------- application ----------

#[test]
fn test_app_simple() {
    let env = empty_env().extend("f".to_string(), t_arr(t_int(), t_bool()));
    assert_type_in(&app(var("f"), num(1)), &env, t_bool());
}

#[test]
fn test_app_applying_non_function() {
    let env = empty_env().extend("x".to_string(), t_int());
    assert_eq!(
        typecheck(&app(var("x"), num(1)), &env),
        Err(TypeError::NotAFunction { actual: t_int() })
    );
}

#[test]
fn test_app_argument_type_mismatch() {
    let env = empty_env().extend("f".to_string(), t_arr(t_int(), t_bool()));
    assert_eq!(
        typecheck(&app(var("f"), Term::True), &env),
        Err(TypeError::ArgTypeMismatch { expected: t_int(), actual: t_bool() })
    );
}

#[test]
fn test_app_left_associative_curried_calls() {
    // f: Int -> Int -> Int, applied as f 1 2
    let env = empty_env().extend("f".to_string(), t_arr(t_int(), t_arr(t_int(), t_int())));
    assert_type_in(&app(app(var("f"), num(1)), num(2)), &env, t_int());
}

#[test]
fn test_app_of_immediately_applied_lambda() {
    // (fun x : Int => x) 5
    assert_type(&app(fun("x", t_int(), var("x")), num(5)), t_int());
}

#[test]
fn test_app_function_position_ill_typed() {
    assert_err(&app(var("undefined"), num(1)), TypeError::UndefinedVariable("undefined".to_string()));
}

// ---------- let ----------

#[test]
fn test_let_simple() {
    assert_type(&let_("x", num(1), var("x")), t_int());
}

#[test]
fn test_let_binding_used_in_arithmetic() {
    assert_type(&let_("x", num(1), bin(BinOp::Add, var("x"), num(2))), t_int());
}

#[test]
fn test_let_body_type_can_differ_from_bound_expr_type() {
    // let x = 1 in true  -- perfectly fine, the overall type is Bool
    assert_type(&let_("x", num(1), Term::True), t_bool());
}

#[test]
fn test_let_bound_expr_ill_typed() {
    assert_err(
        &let_("x", not_(num(1)), var("x")),
        TypeError::InvalidNotOperand { actual: t_int() },
    );
}

#[test]
fn test_let_shadows_outer_binding() {
    let env = empty_env().extend("x".to_string(), t_bool());
    assert_type_in(&let_("x", num(1), var("x")), &env, t_int());
}

#[test]
fn test_let_nested() {
    // let x = 1 in let y = x + 1 in y * 2
    let term = let_(
        "x",
        num(1),
        let_("y", bin(BinOp::Add, var("x"), num(1)), bin(BinOp::Mul, var("y"), num(2))),
    );
    assert_type(&term, t_int());
}

#[test]
fn test_letfun_simple_recursive_call() {
    // letfun f x : Int -> Int = x in f 1
    let term = letfun("f", "x", t_arr(t_int(), t_int()), var("x"), app(var("f"), num(1)));
    assert_type(&term, t_int());
}

#[test]
fn test_letfun_factorial() {
    let term = letfun(
        "fact", "n", t_arr(t_int(), t_int()),
        iff(
            bin(BinOp::Lt, var("n"), num(1)),
            num(1),
            bin(BinOp::Mul, var("n"), app(var("fact"), bin(BinOp::Sub, var("n"), num(1)))),
        ),
        app(var("fact"), num(5)),
    );
    assert_type(&term, t_int());
}

#[test]
fn test_letfun_declared_type_not_arrow() {
    let term = letfun("f", "x", t_int(), var("x"), num(0));
    assert_err(&term, TypeError::NotAFunction { actual: t_int() });
}

#[test]
fn test_letfun_body_does_not_match_declared_return_type() {
    // f : Int -> Int but body returns a Bool
    let term = letfun("f", "x", t_arr(t_int(), t_int()), Term::True, num(0));
    assert_err(
        &term,
        TypeError::LetFunReturnTypeMismatch { expected: t_int(), actual: t_bool() },
    );
}

#[test]
fn test_letfun_body_can_see_itself_and_param() {
    // letfun f x : Int -> Bool = x < 0 in f 5
    let term = letfun(
        "f", "x", t_arr(t_int(), t_bool()),
        bin(BinOp::Lt, var("x"), num(0)),
        app(var("f"), num(5)),
    );
    assert_type(&term, t_bool());
}

#[test]
fn test_letfun_in_term_does_not_see_param() {
    // the function's parameter must not leak into the `in` term's scope
    let term = letfun("f", "x", t_arr(t_int(), t_int()), var("x"), var("x"));
    assert_err(&term, TypeError::UndefinedVariable("x".to_string()));
}

#[test]
fn test_letfun_name_available_in_in_term() {
    let env = empty_env();
    let term = letfun("f", "x", t_arr(t_int(), t_int()), var("x"), var("f"));
    assert_type_in(&term, &env, t_arr(t_int(), t_int()));
}

// ---------- whole-program tests (parse + typecheck together) ----------

#[test]
fn test_program_factorial_types_as_int() {
    let input = r#"
        letfun fact n : Int -> Int =
            if n < 1
            then 1
            else n * fact (n - 1)
        in
        let doubled = fact 5 in
        doubled + 1
    "#;
    assert_program_type(input, t_int());
}

#[test]
fn test_program_higher_order_function_types_as_int() {
    let input = r#"
        let apply_twice = fun f : Int -> Int =>
            fun x : Int => f (f x)
        in
        let inc = fun y : Int => y + 1 in
        apply_twice inc 5
    "#;
    assert_program_type(input, t_int());
}

#[test]
fn test_program_boolean_logic() {
    let input = "let x = 5 in x < 10 && ~false";
    assert_program_type(input, t_bool());
}

#[test]
fn test_program_type_error_undefined_variable() {
    assert_program_err("x + 1", TypeError::UndefinedVariable("x".to_string()));
}

#[test]
fn test_program_type_error_if_condition_not_bool() {
    assert_program_err(
        "if 1 then 2 else 3",
        TypeError::IfConditionMismatch { actual: t_int() },
    );
}

#[test]
fn test_program_type_error_applying_non_function() {
    let input = "let x = 5 in x 1";
    assert_program_err(input, TypeError::NotAFunction { actual: t_int() });
}

#[test]
fn test_program_type_error_wrong_argument_type() {
    let input = "let f = fun x : Int => x + 1 in f true";
    assert_program_err(input, TypeError::ArgTypeMismatch { expected: t_int(), actual: t_bool() });
}
