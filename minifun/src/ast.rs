//! Abstract Syntax Tree (AST) definitions for the 'minifun' language.
//!
//! This module defines the core syntactic structures of the language:
//! - 'BinOp': Arithmetic, comparison, and boolean binary operators, with pretty-printing support.
//! - 'Term': The primary expression grammar, including base literals, variables, control flow,
//!   unary/binary operations, lambda abstractions, applications, and both standard and recursive
//!   let-bindings with optional type annotations.

use crate::typing::types::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, And, Lt
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Lt => write!(f, "<"),
            BinOp::And => write!(f, "&&"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Num(i32),
    True,
    False,
    Var(String),
    BinOp(BinOp, Box<Term>, Box<Term>),
    Not(Box<Term>),
    If(Box<Term>, Box<Term>, Box<Term>),
    Fun(String, Option<Type>, Box<Term>),
    App(Box<Term>, Box<Term>),
    Let(String, Box<Term>, Box<Term>),
    LetFun(String, String, Option<Type>, Box<Term>, Box<Term>),
}