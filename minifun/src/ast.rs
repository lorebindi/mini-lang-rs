use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type{
    Int,
    Bool,
    Arrow(Box<Type>, Box<Type>), // t -> t'
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::Arrow(arg, ret) => {
                // If the argument is itself a function, parentheses are needed
                // because -> is right-associative: (a -> b) -> c
                match **arg {
                    Type::Arrow(_, _) => write!(f, "({arg}) -> {ret}"),
                    _ => write!(f, "{arg} -> {ret}"),
                }
            }
        }
    }
}

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
    // The Type of Fun refers to the type that the argument have to assume in the body
    Fun(String, Type, Box<Term>),
    App(Box<Term>, Box<Term>),
    Let(String, Box<Term>, Box<Term>),
    // The Type of LetFun MUST be a function, i.e.  t -> t' where t is the type
    // of the argument and t' is the type of the resulting term
    LetFun(String, String, Type, Box<Term>, Box<Term>),
}