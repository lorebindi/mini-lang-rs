
use miniimp::parser::parse_program;
use miniimp::ast::*;

fn assert_ast(input: &str, expected: Stmt) {
    let prog = parse_program(input).expect("Parsing fallito");
    assert_eq!(prog.body, expected);
}

// --- ArithExpr ---

#[test]
fn test_assign_num() {
    assert_ast(
        "def main with input x output a as a := 1",
        Stmt::Assign("a".to_string(), ArithExpr::Num(1)),
    );
}

#[test]
fn test_assign_var() {
    assert_ast(
        "def main with input x output a as a := x",
        Stmt::Assign("a".to_string(), ArithExpr::Var("x".to_string())),
    );
}

#[test]
fn test_assign_add() {
    assert_ast(
        "def main with input x output a as a := x + 1",
        Stmt::Assign(
            "a".to_string(),
            ArithExpr::Add(
                Box::new(ArithExpr::Var("x".to_string())),
                Box::new(ArithExpr::Num(1)),
            ),
        ),
    );
}

#[test]
fn test_assign_sub() {
    assert_ast(
        "def main with input x output a as a := x - 1",
        Stmt::Assign(
            "a".to_string(),
            ArithExpr::Sub(
                Box::new(ArithExpr::Var("x".to_string())),
                Box::new(ArithExpr::Num(1)),
            ),
        ),
    );
}

#[test]
fn test_assign_neg_number() {
    assert_ast(
        "def main with input x output a as a := (-1)",
        Stmt::Assign("a".to_string(), ArithExpr::Num(-1)),
    );
}

#[test]
fn test_left_associativity() {
    // x + 1 + 2 => Add(Add(x, 1), 2)
    assert_ast(
        "def main with input x output a as a := x + 1 + 2",
        Stmt::Assign(
            "a".to_string(),
            ArithExpr::Add(
                Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Var("x".to_string())),
                    Box::new(ArithExpr::Num(1)),
                )),
                Box::new(ArithExpr::Num(2)),
            ),
        ),
    );
}

#[test]
fn test_mul_precedence() {
    // x + 2 * 3 => Add(x, Mul(2, 3))
    assert_ast(
        "def main with input x output a as a := x + 2 * 3",
        Stmt::Assign(
            "a".to_string(),
            ArithExpr::Add(
                Box::new(ArithExpr::Var("x".to_string())),
                Box::new(ArithExpr::Mul(
                    Box::new(ArithExpr::Num(2)),
                    Box::new(ArithExpr::Num(3)),
                )),
            ),
        ),
    );
}

// --- BoolExpr ---

#[test]
fn test_bool_true() {
    assert_ast(
        "def main with input x output a as if true then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::True,
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

#[test]
fn test_bool_lt() {
    assert_ast(
        "def main with input x output a as if x < 1 then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::Lt(
                Box::new(ArithExpr::Var("x".to_string())),
                Box::new(ArithExpr::Num(1)),
            ),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

#[test]
fn test_bool_not() {
    assert_ast(
        "def main with input x output a as if not true then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::Not(Box::new(BoolExpr::True)),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

#[test]
fn test_bool_not_not() {
    assert_ast(
        "def main with input x output a as if not not true then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::Not(Box::new(BoolExpr::Not(Box::new(BoolExpr::True)))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

#[test]
fn test_bool_not_paren_lt() {
    assert_ast(
        "def main with input x output a as if not (x < 1) then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::Not(Box::new(BoolExpr::Lt(
                Box::new(ArithExpr::Var("x".to_string())),
                Box::new(ArithExpr::Num(1)),
            ))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

#[test]
fn test_bool_and() {
    assert_ast(
        "def main with input x output a as if true and false then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::And(Box::new(BoolExpr::True), Box::new(BoolExpr::False)),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

#[test]
fn test_bool_and_chain() {
    assert_ast(
        "def main with input x output a as if true and false and true then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::And(
                Box::new(BoolExpr::And(
                    Box::new(BoolExpr::True),
                    Box::new(BoolExpr::False),
                )),
                Box::new(BoolExpr::True),
            ),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

#[test]
fn test_bool_atom_paren() {
    assert_ast(
        "def main with input x output a as if (x < 1) then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::Lt(
                Box::new(ArithExpr::Var("x".to_string())),
                Box::new(ArithExpr::Num(1)),
            ),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

// --- Stmt ---

#[test]
fn test_skip() {
    assert_ast(
        "def main with input x output a as skip",
        Stmt::Skip,
    );
}

#[test]
fn test_seq() {
    assert_ast(
        "def main with input x output a as a := 1 ; a := 2",
        Stmt::Seq(
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(2))),
        ),
    );
}

#[test]
fn test_seq_three() {
    assert_ast(
        "def main with input x output a as a := 1 ; a := 2 ; a := 3",
        Stmt::Seq(
            Box::new(Stmt::Seq(
                Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
                Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(2))),
            )),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(3))),
        ),
    );
}

#[test]
fn test_if_else() {
    assert_ast(
        "def main with input x output a as if x < 5 then a := 1 else a := 0",
        Stmt::If(
            BoolExpr::Lt(
                Box::new(ArithExpr::Var("x".to_string())),
                Box::new(ArithExpr::Num(5)),
            ),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
        ),
    );
}

#[test]
fn test_if_nested_else() {
    assert_ast(
        "def main with input x output a as if true then a := 1 else if false then a := 2 else a := 3",
        Stmt::If(
            BoolExpr::True,
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::If(
                BoolExpr::False,
                Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(2))),
                Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(3))),
            )),
        ),
    );
}

#[test]
fn test_while() {
    assert_ast(
        "def main with input x output a as while x < 1 do skip",
        Stmt::While(
            BoolExpr::Lt(
                Box::new(ArithExpr::Var("x".to_string())),
                Box::new(ArithExpr::Num(1)),
            ),
            Box::new(Stmt::Skip),
        ),
    );
}

#[test]
fn test_bracket() {
    assert_ast(
        "def main with input x output a as (skip)",
        Stmt::Bracket(Box::new(Stmt::Skip)),
    );
}

// --- Programma complesso: fattoriale ---

#[test]
fn test_factorial() {
    assert_ast(
        r#"def main with input x output a as
            a := 1 ;
            (while 1 < x do
                (a := a * x ;
                 x := x + (-1)))"#,
        Stmt::Seq(
            Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::Bracket(Box::new(Stmt::While(
                BoolExpr::Lt(
                    Box::new(ArithExpr::Num(1)),
                    Box::new(ArithExpr::Var("x".to_string())),
                ),
                Box::new(Stmt::Bracket(Box::new(Stmt::Seq(
                    Box::new(Stmt::Assign(
                        "a".to_string(),
                        ArithExpr::Mul(
                            Box::new(ArithExpr::Var("a".to_string())),
                            Box::new(ArithExpr::Var("x".to_string())),
                        ),
                    )),
                    Box::new(Stmt::Assign(
                        "x".to_string(),
                        ArithExpr::Add(
                            Box::new(ArithExpr::Var("x".to_string())),
                            Box::new(ArithExpr::Num(-1)),
                        ),
                    )),
                )))),
            )))),
        ),
    );
}

#[test]
fn test_complex_program() {
    let program = r#"
    def main with input x output y as
        a := x + 1;
        b := 0;
        c := 1;

        while b < 5 do
            (
                c := c * (a + b);
                if not (c < 100) then
                    c := c - 10
                else
                    c := c + 2;
                b := b + 1
            );

        if (c < 50) and (x < 10) then
            (
                y := c + x;
                skip
            )
        else
            (
                y := c - x;
                y := y * 2
            )
"#;

    let expected = Stmt::Seq(
        Box::new(Stmt::Seq(
            Box::new(Stmt::Seq(
                Box::new(Stmt::Seq(
                    // a := x + 1
                    Box::new(Stmt::Assign(
                        "a".to_string(),
                        ArithExpr::Add(
                            Box::new(ArithExpr::Var("x".to_string())),
                            Box::new(ArithExpr::Num(1)),
                        ),
                    )),
                    // b := 0
                    Box::new(Stmt::Assign("b".to_string(), ArithExpr::Num(0))),
                )),
                // c := 1
                Box::new(Stmt::Assign("c".to_string(), ArithExpr::Num(1))),
            )),
            // while b < 5 do (...)
            Box::new(Stmt::While(
                BoolExpr::Lt(
                    Box::new(ArithExpr::Var("b".to_string())),
                    Box::new(ArithExpr::Num(5)),
                ),
                Box::new(Stmt::Bracket(Box::new(Stmt::Seq(
                    Box::new(Stmt::Seq(
                        // c := c * (a + b)
                        Box::new(Stmt::Assign(
                            "c".to_string(),
                            ArithExpr::Mul(
                                Box::new(ArithExpr::Var("c".to_string())),
                                Box::new(ArithExpr::Add(
                                    Box::new(ArithExpr::Var("a".to_string())),
                                    Box::new(ArithExpr::Var("b".to_string())),
                                )),
                            ),
                        )),
                        // if not (c < 100) then c := c - 10 else c := c + 2
                        Box::new(Stmt::If(
                            BoolExpr::Not(Box::new(BoolExpr::Lt(
                                Box::new(ArithExpr::Var("c".to_string())),
                                Box::new(ArithExpr::Num(100)),
                            ))),
                            Box::new(Stmt::Assign(
                                "c".to_string(),
                                ArithExpr::Sub(
                                    Box::new(ArithExpr::Var("c".to_string())),
                                    Box::new(ArithExpr::Num(10)),
                                ),
                            )),
                            Box::new(Stmt::Assign(
                                "c".to_string(),
                                ArithExpr::Add(
                                    Box::new(ArithExpr::Var("c".to_string())),
                                    Box::new(ArithExpr::Num(2)),
                                ),
                            )),
                        )),
                    )),
                    // b := b + 1
                    Box::new(Stmt::Assign(
                        "b".to_string(),
                        ArithExpr::Add(
                            Box::new(ArithExpr::Var("b".to_string())),
                            Box::new(ArithExpr::Num(1)),
                        ),
                    )),
                )))),
            )),
        )),
        // if (c < 50) and (x < 10) then (...) else (...)
        Box::new(Stmt::If(
            BoolExpr::And(
                Box::new(BoolExpr::Lt(
                    Box::new(ArithExpr::Var("c".to_string())),
                    Box::new(ArithExpr::Num(50)),
                )),
                Box::new(BoolExpr::Lt(
                    Box::new(ArithExpr::Var("x".to_string())),
                    Box::new(ArithExpr::Num(10)),
                )),
            ),
            Box::new(Stmt::Bracket(Box::new(Stmt::Seq(
                Box::new(Stmt::Assign(
                    "y".to_string(),
                    ArithExpr::Add(
                        Box::new(ArithExpr::Var("c".to_string())),
                        Box::new(ArithExpr::Var("x".to_string())),
                    ),
                )),
                Box::new(Stmt::Skip),
            )))),
            Box::new(Stmt::Bracket(Box::new(Stmt::Seq(
                Box::new(Stmt::Assign(
                    "y".to_string(),
                    ArithExpr::Sub(
                        Box::new(ArithExpr::Var("c".to_string())),
                        Box::new(ArithExpr::Var("x".to_string())),
                    ),
                )),
                Box::new(Stmt::Assign(
                    "y".to_string(),
                    ArithExpr::Mul(
                        Box::new(ArithExpr::Var("y".to_string())),
                        Box::new(ArithExpr::Num(2)),
                    ),
                )),
            )))),
        )),
    );

    assert_ast(program, expected);
}

#[test]
fn test_keyword_prefix_in_bool_context() {
    // "truevalue" MUST not interfere with the if condition
    assert_ast(
        "def main with input x output a as truevalue := 1; if truevalue < 1 then a := 1 else a := 0",
        Stmt::Seq(
            Box::new(Stmt::Assign("truevalue".to_string(), ArithExpr::Num(1))),
            Box::new(Stmt::If(
                BoolExpr::Lt(
                    Box::new(ArithExpr::Var("truevalue".to_string())),
                    Box::new(ArithExpr::Num(1)),
                ),
                Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(1))),
                Box::new(Stmt::Assign("a".to_string(), ArithExpr::Num(0))),
            )),
        ),
    );
}

#[test]
fn test_keyword_prefix_ident2() {
    // "noter" MUST not interfere with "not"
    assert_ast(
        "def main with input x output a as noter := 1",
        Stmt::Assign("noter".to_string(), ArithExpr::Num(1)),
    );
}

#[test]
fn test_keyword_prefix_ident3() {
    // "android" MUST not interfere with "and"
    assert_ast(
        "def main with input x output a as android := 1",
        Stmt::Assign("android".to_string(), ArithExpr::Num(1)),
    );
}

#[test]
fn test_parse_error_missing_else() {
    assert!(parse_program("def main with input x output a as if true then a := 1").is_none());
}

#[test]
fn test_parse_error_missing_output() {
    assert!(parse_program("def main with input x as skip").is_none());
}

#[test]
fn test_parse_error_empty_body() {
    assert!(parse_program("def main with input x output a as").is_none());
}
