use minifun::parser::*;
use minifun::ast::*;

fn assert_ast(input: &str, expected: Term) {
    let term = parse_term(input).expect("Parsing fallito");
    assert_eq!(term, expected);
}

// Atoms

#[test]
fn test_num() {
    assert_ast("42", Term::Num(42));
}

#[test]
fn test_negative_num() {
    assert_ast("-5", Term::Num(-5));
}

#[test]
fn test_true() {
    assert_ast("true", Term::True);
}

#[test]
fn test_false() {
    assert_ast("false", Term::False);
}

#[test]
fn test_var() {
    assert_ast("x", Term::Var("x".to_string()));
}

// Arithmetic operator

#[test]
fn test_add() {
    assert_ast(
        "x + 1",
        Term::BinOp(
            BinOp::Add,
            Box::new(Term::Var("x".to_string())),
            Box::new(Term::Num(1)),
        ),
    );
}

#[test]
fn test_sub() {
    assert_ast(
        "x - 1",
        Term::BinOp(
            BinOp::Sub,
            Box::new(Term::Var("x".to_string())),
            Box::new(Term::Num(1)),
        ),
    );
}

#[test]
fn test_mul() {
    assert_ast(
        "x * 2",
        Term::BinOp(
            BinOp::Mul,
            Box::new(Term::Var("x".to_string())),
            Box::new(Term::Num(2)),
        ),
    );
}

#[test]
fn test_left_associativity_add() {
    // x + 1 + 2 => Add(Add(x, 1), 2)
    assert_ast(
        "x + 1 + 2",
        Term::BinOp(
            BinOp::Add,
            Box::new(Term::BinOp(
                BinOp::Add,
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Num(1)),
            )),
            Box::new(Term::Num(2)),
        ),
    );
}

#[test]
fn test_mul_precedence() {
    // x + 2 * 3 => Add(x, Mul(2, 3))
    assert_ast(
        "x + 2 * 3",
        Term::BinOp(
            BinOp::Add,
            Box::new(Term::Var("x".to_string())),
            Box::new(Term::BinOp(
                BinOp::Mul,
                Box::new(Term::Num(2)),
                Box::new(Term::Num(3)),
            )),
        ),
    );
}

#[test]
fn test_parentheses_override_precedence() {
    // (x + 2) * 3 => Mul(Add(x, 2), 3)
    assert_ast(
        "(x + 2) * 3",
        Term::BinOp(
            BinOp::Mul,
            Box::new(Term::BinOp(
                BinOp::Add,
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Num(2)),
            )),
            Box::new(Term::Num(3)),
        ),
    );
}

#[test]
fn test_left_associativity_mul() {
    // x * 2 * 3 => Mul(Mul(x, 2), 3)
    assert_ast(
        "x * 2 * 3",
        Term::BinOp(
            BinOp::Mul,
            Box::new(Term::BinOp(
                BinOp::Mul,
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Num(2)),
            )),
            Box::new(Term::Num(3)),
        ),
    );
}

#[test]
fn test_subtract_negative_literal() {
    // x - -3 => Sub(x, -3)
    assert_ast(
        "x - -3",
        Term::BinOp(
            BinOp::Sub,
            Box::new(Term::Var("x".to_string())),
            Box::new(Term::Num(-3)),
        ),
    );
}

#[test]
fn test_no_whitespace_between_tokens() {
    assert_ast(
        "x+1",
        Term::BinOp(
            BinOp::Add,
            Box::new(Term::Var("x".to_string())),
            Box::new(Term::Num(1)),
        ),
    );
}

// Logic operator and comparison

#[test]
fn test_lt() {
    assert_ast(
        "x < 1",
        Term::BinOp(
            BinOp::Lt,
            Box::new(Term::Var("x".to_string())),
            Box::new(Term::Num(1)),
        ),
    );
}

#[test]
fn test_chained_comparison_is_error() {
    assert!(parse_term("x < 1 < 2").is_none());
}

#[test]
fn test_not() {
    assert_ast(
        "~true",
        Term::Not(Box::new(Term::True)),
    );
}

#[test]
fn test_not_not() {
    assert_ast(
        "~~true",
        Term::Not(Box::new(
            Term::Not(Box::new(Term::True)),
        )),
    );
}

#[test]
fn test_and() {
    assert_ast(
        "true && false",
        Term::BinOp(
            BinOp::And,
            Box::new(Term::True),
            Box::new(Term::False),
        ),
    );
}

#[test]
fn test_and_chain() {
    // true && false && true => And(And(true, false), true)
    assert_ast(
        "true && false && true",
        Term::BinOp(
            BinOp::And,
            Box::new(Term::BinOp(
                BinOp::And,
                Box::new(Term::True),
                Box::new(Term::False),
            )),
            Box::new(Term::True),
        ),
    );
}

#[test]
fn test_operator_precedence() {
    // x < 5 && true => And(Lt(x, 5), true)
    assert_ast(
        "x < 5 && true",
        Term::BinOp(
            BinOp::And,
            Box::new(Term::BinOp(
                BinOp::Lt,
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Num(5)),
            )),
            Box::new(Term::True),
        ),
    );
}

// If

#[test]
fn test_if_else() {
    assert_ast(
        "if true then 1 else 0",
        Term::If(
            Box::new(Term::True),
            Box::new(Term::Num(1)),
            Box::new(Term::Num(0)),
        ),
    );
}

#[test]
fn test_if_nested() {
    assert_ast(
        "if true then 1 else if false then 2 else 3",
        Term::If(
            Box::new(Term::True),
            Box::new(Term::Num(1)),
            Box::new(Term::If(
                Box::new(Term::False),
                Box::new(Term::Num(2)),
                Box::new(Term::Num(3)),
            )),
        ),
    );
}

// Function and application

#[test]
fn test_fun() {
    assert_ast(
        "fun x => x",
        Term::Fun(
            "x".to_string(),
            Box::new(Term::Var("x".to_string())),
        ),
    );
}

#[test]
fn test_application() {
    assert_ast(
        "f x",
        Term::App(
            Box::new(Term::Var("f".to_string())),
            Box::new(Term::Var("x".to_string())),
        ),
    );
}

#[test]
fn test_application_left_associativity() {
    // f x y => App(App(f, x), y)
    assert_ast(
        "f x y",
        Term::App(
            Box::new(Term::App(
                Box::new(Term::Var("f".to_string())),
                Box::new(Term::Var("x".to_string())),
            )),
            Box::new(Term::Var("y".to_string())),
        ),
    );
}

#[test]
fn test_apply_function_expression() {
    // (fun x => x) 5
    assert_ast(
        "(fun x => x) 5",
        Term::App(
            Box::new(Term::Fun(
                "x".to_string(),
                Box::new(Term::Var("x".to_string())),
            )),
            Box::new(Term::Num(5)),
        ),
    );
}

#[test]
fn test_application_binds_tighter_than_add() {
    // f x + 1 => Add(App(f, x), 1), not App(f, x + 1)
    assert_ast(
        "f x + 1",
        Term::BinOp(
            BinOp::Add,
            Box::new(Term::App(
                Box::new(Term::Var("f".to_string())),
                Box::new(Term::Var("x".to_string())),
            )),
            Box::new(Term::Num(1)),
        ),
    );
}

#[test]
fn test_application_with_parenthesized_argument() {
    // f (x + 1) => App(f, Add(x, 1))
    assert_ast(
        "f (x + 1)",
        Term::App(
            Box::new(Term::Var("f".to_string())),
            Box::new(Term::BinOp(
                BinOp::Add,
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Num(1)),
            )),
        ),
    );
}

#[test]
fn test_not_binds_application_as_whole() {
    // ~f x => Not(App(f, x)), not App(Not(f), x)
    assert_ast(
        "~f x",
        Term::Not(Box::new(Term::App(
            Box::new(Term::Var("f".to_string())),
            Box::new(Term::Var("x".to_string())),
        ))),
    );
}

// Let
#[test]
fn test_let() {
    assert_ast(
        "let x = 1 in x",
        Term::Let(
            "x".to_string(),
            Box::new(Term::Num(1)),
            Box::new(Term::Var("x".to_string())),
        ),
    );
}

// LetFun

#[test]
fn test_letfun() {
    assert_ast(
        "letfun f x = x + 1 in f 2",
        Term::LetFun(
            "f".to_string(),
            "x".to_string(),
            Box::new(Term::BinOp(
                BinOp::Add,
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Num(1)),
            )),
            Box::new(Term::App(
                Box::new(Term::Var("f".to_string())),
                Box::new(Term::Num(2)),
            )),
        ),
    );
}

// Parsing errors

#[test]
fn test_parse_error_missing_else() {
    assert!(
        parse_term("if true then 1").is_none()
    );
}

#[test]
fn test_parse_error_missing_in() {
    assert!(
        parse_term("let x = 1 x").is_none()
    );
}

#[test]
fn test_parse_error_empty_input() {
    assert!(
        parse_term("").is_none()
    );
}

#[test]
fn test_parse_error_unclosed_paren() {
    assert!(parse_term("(x + 1").is_none());
}

#[test]
fn test_parse_error_fun_missing_arrow() {
    assert!(parse_term("fun x x").is_none());
}

#[test]
fn test_parse_error_trailing_garbage() {
    // valid expr followed by an unconsumed token — EOI should reject this
    assert!(parse_term("1 1 )").is_none());
}

// identifiers as substring

fn test_var_prefixed_with_keyword() {
    assert_ast("iffy", Term::Var("iffy".to_string()));
}

#[test]
fn test_var_containing_keyword_words() {
    assert_ast("thenable", Term::Var("thenable".to_string()));
    assert_ast("elsewhere", Term::Var("elsewhere".to_string()));
    assert_ast("letter", Term::Var("letter".to_string()));
    assert_ast("funny", Term::Var("funny".to_string()));
}

#[test]
fn test_keyword_as_identifier_is_rejected() {
    // "if" alone can't be a variable
    assert!(parse_term("let if = 1 in if").is_none());
}

