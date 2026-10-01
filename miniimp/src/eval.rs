//! Big-step operational semantics interpreter and expression evaluator for 'miniimp'.
//!
//! This module implements direct execution over the Abstract Syntax Tree by maintaining
//! an evaluation 'Environment':
//! - 'eval_arith': Evaluates arithmetic expressions ('ArithExpr') to 64-bit signed integers ('i64'),
//!   resolving bound identifiers through the environment.
//! - 'eval_bool': Evaluates boolean expressions and comparisons ('BoolExpr') to 'bool'.
//! - 'eval_stmt': Executes commands ('Cmd') by mutating program state, handling sequential
//!   composition, conditionals, and standard 'while' loops.

use crate::ast::*;
use crate::enviroment::*;

pub fn eval_arith(expr: &ArithExpr, env: &Environment) -> i64 {
    match expr {
        ArithExpr::Num(n) => *n,
        ArithExpr::Var(x) => env.lookup(x),
        ArithExpr::Add(e1, e2) => eval_arith(e1, env) + eval_arith(e2, env),
        ArithExpr::Sub(e1, e2) => eval_arith(e1, env) - eval_arith(e2, env),
        ArithExpr::Mul(e1, e2) => eval_arith(e1, env) * eval_arith(e2, env),
    }
}

pub fn eval_bool(expr: &BoolExpr, env: &Environment) -> bool {
    match expr {
        BoolExpr::True => true,
        BoolExpr::False => false,
        BoolExpr::And(e1, e2) => {eval_bool(e1, env) && eval_bool(e2, env)},
        BoolExpr::Lt(e1, e2) => {eval_arith(e1, env) < eval_arith(e2, env)},
        BoolExpr::Not(e) => !eval_bool(e, env)
    }
}

pub fn eval_stmt(stmt: &Cmd, env: &mut Environment)  {
    match stmt {
        Cmd::Skip => (),
        Cmd::Assign(var, value) => env.update(var.to_string(), eval_arith(value, env)),
        Cmd::Seq(c1, c2) => {
            eval_stmt(c1, env);
            eval_stmt(c2, env);
        },
        Cmd::If(cond, c1, c2 ) => {
            if eval_bool(cond, env) {
                eval_stmt(c1, env);
            } else {
                eval_stmt(c2, env);
            }
        }
        Cmd::While(cond, c) => {
            while eval_bool(cond, env) {
                eval_stmt(c, env);
            }
        }
        Cmd::Bracket(c) => eval_stmt(c, env),
    }
}