//! Abstract Syntax Tree (AST) definitions for the 'miniimp' language.
//!
//! Provides the data structures representing parsed source terms before evaluation
//! or translation into intermediate representations:
//! - 'ArithExpr': Arithmetic expressions yielding signed 64-bit integers ('i64'),
//!   including constants, variable lookups, and basic binary operations ('+', '-', '*').
//! - 'BoolExpr': Boolean formulas and conditions evaluated for control branching,
//!   supporting truth literals, logical operators ('not', 'and'), and comparisons ('<').
//! - 'Cmd': Imperative commands constituting the executable body, such as assignments,
//!   sequential composition (';'), conditional execution ('if'), loops ('while'), and 'skip'.
//! - 'Prog': Top-level program representation capturing input/output variable declarations
//!   alongside the root command body.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArithExpr {
    Num(i64),
    Var(String),
    Add(Box<ArithExpr>, Box<ArithExpr>),
    Sub(Box<ArithExpr>, Box<ArithExpr>),
    Mul(Box<ArithExpr>, Box<ArithExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoolExpr {
    True,
    False,
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Lt(Box<ArithExpr>, Box<ArithExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    Bracket(Box<Cmd>), // (cmd)
    Skip,
    Assign(String, ArithExpr),
    Seq(Box<Cmd>, Box<Cmd>),
    If(BoolExpr, Box<Cmd>, Box<Cmd>),
    While(BoolExpr, Box<Cmd>),
}
#[derive(Debug, Clone)]
pub struct Prog {
    pub input: String,
    pub output: String,
    pub body: Cmd,
}