#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, And, Lt
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
    Fun(String, Box<Term>),
    App(Box<Term>, Box<Term>),
    Let(String, Box<Term>, Box<Term>),
    LetFun(String, String, Box<Term>, Box<Term>),
}