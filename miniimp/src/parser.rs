use pest_derive::Parser;
use pest::iterators::Pair;
use crate::ast::*;
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MiniImpParser;

/*
* Builds an arithmetic expressione (ArithExpr) from a pest's parsing
* tree node (Pair). Handles numbers, variables, binary operations
(+, -, *) and nested expressions.
*/
pub fn build_arith_expr(pair: Pair<Rule>) -> ArithExpr {
    match pair.as_rule() {
        Rule::number => {
            ArithExpr::Num(pair.as_str().parse().unwrap())
        }

        Rule::ident => {
            ArithExpr::Var(pair.as_str().to_string())
        }

        Rule::expr => {
            let mut inner = pair.into_inner();
            let mut left = build_arith_expr(inner.next().unwrap());

            while let Some(op) = inner.next() {
                let right = build_arith_expr(inner.next().unwrap());

                left = match op.as_str() {
                    "+" => ArithExpr::Add(Box::new(left), Box::new(right)),
                    "-" => ArithExpr::Sub(Box::new(left), Box::new(right)),
                    _ => unreachable!(),
                };
            }

            left
        }

        Rule::term => {
            let mut inner = pair.into_inner();
            let mut left = build_arith_expr(inner.next().unwrap());

            while let Some(op) = inner.next() {
                let right = build_arith_expr(inner.next().unwrap());

                left = match op.as_str() {
                    "*" => ArithExpr::Mul(Box::new(left), Box::new(right)),
                    _ => unreachable!(),
                };
            }

            left
        }

        Rule::factor => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();

            match first.as_rule() {
                Rule::number => ArithExpr::Num(first.as_str().parse().unwrap()),

                Rule::ident => ArithExpr::Var(first.as_str().to_string()),

                Rule::neg_number => {
                    let num = first.into_inner().next().unwrap();
                    let value: i64 = num.as_str().parse().unwrap();
                    ArithExpr::Num(-value)
                }

                Rule::expr => build_arith_expr(first),

                _ => unreachable!(),
            }
        }

        _ => unreachable!("Unexpected rule in ArithExpr"),
    }
}

/*
* Builds a boolean expression (`BoolExpr`) from a parsing tree node.
* Supports logical operators (and, not), comparisons (<), and boolean
* literals.
*/
pub fn build_bool_expr(pair: Pair<Rule>) -> BoolExpr {

    match pair.as_rule() {
        Rule::bool_expr => build_bool_expr(pair.into_inner().next().unwrap()),

        Rule::bool_and => {
            let mut inner = pair.into_inner();
            let mut left = build_bool_expr(inner.next().unwrap());

            while let Some(_and_op) = inner.next() { // consuma and_op
                let right = build_bool_expr(inner.next().unwrap());
                left = BoolExpr::And(Box::new(left), Box::new(right));
            }

            left
        }

        Rule::bool_not => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();

            match first.as_rule() {
                Rule::not_op => {
                    let expr = build_bool_expr(inner.next().unwrap());
                    BoolExpr::Not(Box::new(expr))
                }
                Rule::bool_atom => build_bool_expr(first),
                _ => unreachable!(),
            }
        }
        Rule::bool_atom => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();

            match first.as_rule() {
                Rule::bool_true => BoolExpr::True,
                Rule::bool_false => BoolExpr::False,
                Rule::expr => {
                    let left = build_arith_expr(first);
                    let right = build_arith_expr(inner.next().unwrap());
                    BoolExpr::Lt(Box::new(left), Box::new(right))
                }
                Rule::bool_atom_bracket => {
                    build_bool_expr(first.into_inner().next().unwrap())
                }
                _ => unreachable!(),
            }
        }

        _ => unreachable!(),
    }
}

/*
* Builds a statement (`Stmt`) of the MiniImp language from a parsing node.
* Handles assignments, sequences, conditionals (if), loops (while),
* skip statements, and bracketed command blocks.
*/
fn build_stmt(pair: Pair<Rule>) -> Stmt {
    match pair.as_rule() {
        Rule::stmt => build_stmt(pair.into_inner().next().unwrap()),

        Rule::cmd => {
            let mut inner = pair.into_inner();
            let first = build_stmt(inner.next().unwrap());
            inner.fold(first, |acc, next| {
                Stmt::Seq(Box::new(acc), Box::new(build_stmt(next)))
            })
        }

        Rule::assign => {
            let mut inner = pair.into_inner();
            let ident = inner.next().unwrap().as_str().to_string();
            let expr = build_arith_expr(inner.next().unwrap());
            Stmt::Assign(ident, expr)
        }

        Rule::skip => Stmt::Skip,

        Rule::if_stmt => {
            let mut inner = pair.into_inner();
            let cond = build_bool_expr(inner.next().unwrap());
            let then_branch = build_stmt(inner.next().unwrap());
            let else_branch = build_stmt(inner.next().unwrap());
            Stmt::If(cond, Box::new(then_branch), Box::new(else_branch))
        }

        Rule::while_stmt => {
            let mut inner = pair.into_inner();
            let cond = build_bool_expr(inner.next().unwrap());
            let body = build_stmt(inner.next().unwrap());
            Stmt::While(cond, Box::new(body))
        }

        Rule::bracket_cmd => {
            let inner = pair.into_inner().next().unwrap();
            Stmt::Bracket(Box::new(build_stmt(inner)))
        }

        _ => unreachable!("Unexpected rule in Stmt: {:?}", pair.as_rule()),
    }
}

/*
* Parses a complete MiniImp program from the input string.
* Extracts the input variable, output variable, and program body.
* Returns `Some(Prog)` if parsing succeeds, otherwise `None`.
*/
pub fn parse_program(input: &str) -> Option<Prog>{

    match <MiniImpParser as pest::Parser<Rule>>::parse(Rule::prog, input) {
        Ok(mut pairs) => {
            let prog_pair = pairs.next().unwrap();
            let mut inner = prog_pair.into_inner();

            let input_var = inner.next().unwrap().as_str().to_string();  // ident dopo "input"
            let output_var = inner.next().unwrap().as_str().to_string(); // ident dopo "output"
            let body = build_stmt(inner.next().unwrap());                // cmd

            Some(Prog { input: input_var, output: output_var, body })
        }
        Err(e) => {
            println!("Errore di parsing: {}", e);
            None
        }
    }
}