// Shared helpers for tests/parser_tests.rs and tests/typecheck_tests.rs.
//
// This lives at tests/common/mod.rs (not tests/common.rs) specifically so
// cargo does NOT treat it as its own standalone test binary — only direct
// tests/*.rs files are compiled as separate test crates. Files inside a
// subdirectory are plain modules that other test files can `mod` in.

#![allow(dead_code)] // not every test file uses every helper

use minifun::ast::*;
use minifun::typing::*;

pub fn t_int() -> Type { Type::Int }
pub fn t_bool() -> Type { Type::Bool }
pub fn t_arr(from: Type, to: Type) -> Type { Type::Arrow(Box::new(from), Box::new(to)) }

pub fn num(n: i32) -> Term { Term::Num(n) }
pub fn var(s: &str) -> Term { Term::Var(s.to_string()) }
pub fn bin(op: BinOp, l: Term, r: Term) -> Term { Term::BinOp(op, Box::new(l), Box::new(r)) }
pub fn app(f: Term, a: Term) -> Term { Term::App(Box::new(f), Box::new(a)) }
pub fn not_(t: Term) -> Term { Term::Not(Box::new(t)) }
pub fn iff(c: Term, t: Term, e: Term) -> Term { Term::If(Box::new(c), Box::new(t), Box::new(e)) }
pub fn fun(p: &str, t: Type, b: Term) -> Term { Term::Fun(p.to_string(), Some(t), Box::new(b)) }
pub fn let_(v: &str, val: Term, body: Term) -> Term {
    Term::Let(v.to_string(), Box::new(val), Box::new(body))
}
pub fn letfun(f: &str, p: &str, t: Type, body: Term, in_: Term) -> Term {
    Term::LetFun(f.to_string(), p.to_string(), Some(t), Box::new(body), Box::new(in_))
}

pub fn fun_unannotated(p: &str, b: Term) -> Term {
    Term::Fun(p.to_string(), None, Box::new(b))
}
pub fn letfun_unannotated(f: &str, p: &str, body: Term, in_: Term) -> Term {
    Term::LetFun(f.to_string(), p.to_string(), None, Box::new(body), Box::new(in_))
}