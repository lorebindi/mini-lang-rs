#[derive(Debug, Clone, PartialEq)]
pub enum ArithExpr {
    Num(i64),
    Var(String),
    Add(Box<ArithExpr>, Box<ArithExpr>),
    Sub(Box<ArithExpr>, Box<ArithExpr>),
    Mul(Box<ArithExpr>, Box<ArithExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolExpr {
    True,
    False,
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Lt(Box<ArithExpr>, Box<ArithExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Bracket(Box<Stmt>),
    Skip,
    Assign(String, ArithExpr),
    Seq(Box<Stmt>, Box<Stmt>),
    If(BoolExpr, Box<Stmt>, Box<Stmt>),
    While(BoolExpr, Box<Stmt>),
}
#[derive(Debug, Clone)]
pub struct Prog {
    pub input: String,
    pub output: String,
    pub body: Stmt,
}