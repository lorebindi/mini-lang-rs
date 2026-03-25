use pest_derive::Parser;

use pest::iterators::Pair;
use crate::ast::*;
#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MiniImpParser;

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

/*pub fn build_bool_expr(pair: Pair<Rule>) -> BoolExpr {

}*/

/*
fn build_stmt(pair: Pair<Rule>) -> Stmt {
    match pair.as_rule() {
        Rule::assign => {
            let mut inner = pair.into_inner();
            let ident = inner.next().unwrap().as_str().to_string();
            let expr = build_ArithExpr(inner.next().unwrap());
            Stmt::Assign(ident, expr)
        }

        Rule::if_stmt => {
            let mut inner = pair.into_inner();

            let cond = build_BoolExpr(inner.next().unwrap());
            let then_branch = build_stmt(inner.next().unwrap());
            let else_branch = build_stmt(inner.next().unwrap());

            Stmt::If(
                cond,
                Box::new(then_branch),
                Box::new(else_branch),
            )
        }

        Rule::while_stmt => {
            let mut inner = pair.into_inner();

            let cond = build_BoolExpr(inner.next().unwrap());
            let body = build_stmt(inner.next().unwrap());

            Stmt::While(cond, Box::new(body))
        }

        _ => unreachable!(),
    }
}*/

pub fn parse_program(input: &str) {
    match <MiniImpParser as pest::Parser<Rule>>::parse(Rule::prog, input) {
        Ok(pairs) => {
            println!("Parsing riuscito!\n");

            for pair in pairs {
                println!("{:#?}", pair);
            }
        }
        Err(e) => {
            println!("Errore di parsing:");
            println!("{}", e);
        }
    }
}