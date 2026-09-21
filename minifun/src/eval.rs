use std::rc::Rc;

use crate::ast::*;
use crate::scope::Scope;

pub type Environment = Scope<Value>;

#[derive(Clone)]
pub struct Closure {
    pub param: String,
    pub body: Term,
    pub env: Rc<Environment>,
}

#[derive(Clone)]
pub enum Value {
    Num(i32),
    Bool(bool),
    Closure(Closure),
}