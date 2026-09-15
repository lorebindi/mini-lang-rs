use crate::ast::*;
use crate::enviroment::*;

pub fn eval_arith(expr: &ArithExpr, env: &Env) -> i64 {
    match expr {
        ArithExpr::Num(n) => *n,
        ArithExpr::Var(x) => lookup(env, x),
        ArithExpr::Add(e1, e2) => eval_arith(e1, env) + eval_arith(e2, env),
        ArithExpr::Sub(e1, e2) => eval_arith(e1, env) - eval_arith(e2, env),
        ArithExpr::Mul(e1, e2) => eval_arith(e1, env) * eval_arith(e2, env),
    }
}

pub fn eval_bool(expr: &BoolExpr, env: &Env) -> bool {
    match expr {
        BoolExpr::True => true,
        BoolExpr::False => false,
        BoolExpr::And(e1, e2) => {eval_bool(e1, env) && eval_bool(e2, env)},
        BoolExpr::Lt(e1, e2) => {eval_arith(e1, env) < eval_arith(e2, env)},
        BoolExpr::Not(e) => !eval_bool(e, env)
    }
}

pub fn eval_stmt(stmt: &Stmt, env: &mut Env)  {
    match stmt {
        Stmt::Skip => (),
        Stmt::Assign(var, value) => update(env, var.to_string(), eval_arith(value, env)),
        Stmt::Seq(c1, c2) => {
            eval_stmt(c1, env);
            eval_stmt(c2, env);
        },
        Stmt::If(cond, c1, c2 ) => {
            if eval_bool(cond, env) {
                eval_stmt(c1, env);
            } else {
                eval_stmt(c2, env);
            }
        }
        Stmt::While(cond, c) => {
            while eval_bool(cond, env) {
                eval_stmt(c, env);
            }
        }
        Stmt::Bracket(c) => eval_stmt(c, env),
    }
}