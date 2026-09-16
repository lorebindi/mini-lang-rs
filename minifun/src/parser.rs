use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use crate::ast::{BinOp, Term};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MiniFunParser;

fn build_atom(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::number => {
            Term::Num(first.as_str().parse().unwrap())
        }

        Rule::kw_true => {
            Term::True
        }

        Rule::kw_false => {
            Term::False
        }

        Rule::ident => {
            Term::Var(first.as_str().to_string())
        }

        Rule::expr => {
            build_expr(first)
        }

        _ => unreachable!(),
    }
}

fn build_application(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    let mut left = build_atom(inner.next().unwrap());

    while let Some(atom) = inner.next() {
        let right = build_atom(atom);

        left = Term::App(
            Box::new(left),
            Box::new(right),
        );
    }

    left
}

fn build_unary_expr(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::not_expr => {
            let expr = build_unary_expr(
                first.into_inner().next().unwrap()
            );

            Term::Not(Box::new(expr))
        }

        Rule::application => build_application(first),

        _ => unreachable!(),
    }
}

fn build_add_expr(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    let mut left = build_mul_expr(inner.next().unwrap());

    while let Some(op) = inner.next() {
        let right = build_mul_expr(inner.next().unwrap());

        left = match op.as_str() {
            "+" => Term::BinOp(
                BinOp::Add,
                Box::new(left),
                Box::new(right),
            ),

            "-" => Term::BinOp(
                BinOp::Sub,
                Box::new(left),
                Box::new(right),
            ),

            _ => unreachable!(),
        };
    }

    left
}

fn build_mul_expr(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    let mut left = build_unary_expr(inner.next().unwrap());

    while let Some(_op) = inner.next() {
        let right = build_unary_expr(inner.next().unwrap());

        left = Term::BinOp(
            BinOp::Mul,
            Box::new(left),
            Box::new(right),
        );
    }

    left
}

fn build_comp_expr(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    let mut left = build_add_expr(inner.next().unwrap());

    if let Some(_lt_op) = inner.next() {
        let right = build_add_expr(inner.next().unwrap());

        left = Term::BinOp(
            BinOp::Lt,
            Box::new(left),
            Box::new(right),
        );
    }

    left
}

fn build_logic_expr(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    let mut left = build_comp_expr(inner.next().unwrap());

    while let Some(_and_op) = inner.next() {
        let right = build_comp_expr(inner.next().unwrap());

        left = Term::BinOp(
            BinOp::And,
            Box::new(left),
            Box::new(right),
        );
    }

    left
}

fn build_fun(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    inner.next(); // skip kw_fun
    let parameter = inner.next().unwrap().as_str().to_string();
    let body = build_expr(inner.next().unwrap());

    Term::Fun(parameter, Box::new(body))
}

fn build_if(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    inner.next(); // skip kw_if
    let condition = build_expr(inner.next().unwrap());
    inner.next(); // skip kw_then
    let then_branch = build_expr(inner.next().unwrap());
    inner.next(); // skip kw_else
    let else_branch = build_expr(inner.next().unwrap());

    Term::If(
        Box::new(condition),
        Box::new(then_branch),
        Box::new(else_branch),
    )
}

fn build_let(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    inner.next(); // skip kw_let
    let variable = inner.next().unwrap().as_str().to_string();
    let value = build_expr(inner.next().unwrap());
    inner.next(); // skip kw_in
    let body = build_expr(inner.next().unwrap());

    Term::Let(
        variable,
        Box::new(value),
        Box::new(body),
    )
}

fn build_letfun(pair: Pair<Rule>) -> Term {
    let mut inner = pair.into_inner();

    inner.next(); // skip kw_letfun
    let function_name = inner.next().unwrap().as_str().to_string();
    let parameter = inner.next().unwrap().as_str().to_string();
    let function_body = build_expr(inner.next().unwrap());
    inner.next(); // skip kw_in
    let in_expr = build_expr(inner.next().unwrap());

    Term::LetFun(
        function_name,
        parameter,
        Box::new(function_body),
        Box::new(in_expr),
    )
}

fn build_expr(pair: Pair<Rule>) -> Term {
    match pair.as_rule() {
        Rule::expr => build_expr(pair.into_inner().next().unwrap()),
        Rule::letfun_expr => build_letfun(pair),
        Rule::let_expr => build_let(pair),
        Rule::if_expr => build_if(pair),
        Rule::fun_expr => build_fun(pair),
        Rule::logic_expr => build_logic_expr(pair),
        _ => unreachable!("Unexpected rule in Expr: {:?}", pair.as_rule()),
    }
}

pub fn parse_term(input: &str) -> Option<Term> {
    match <MiniFunParser as pest::Parser<Rule>>::parse(Rule::term, input) {
        Ok(mut pairs) => {
            let term_pair = pairs.next().unwrap();
            let expr_pair = term_pair.into_inner().next().unwrap();

            Some(build_expr(expr_pair))
        }
        Err(e) => {
            println!("Errore di parsing: {}", e);
            None
        }
    }
}