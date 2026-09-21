use std::fmt;
use crate::ast::*;
use crate::scope::Scope;

pub type Context = Scope<Type>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperandPosition {
    Left,
    Right,
}

impl fmt::Display for OperandPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperandPosition::Left => write!(f, "left"),
            OperandPosition::Right => write!(f, "right"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    // Reference to an identifier not present in the context
    UndefinedVariable(String),

    // Expected type differs from the type actually found
    BinOpTypeMismatch {op:BinOp, position: OperandPosition, expected: Type, actual: Type, },

    // Expected type to apply Not operand differs from the type actually found
    InvalidNotOperand {actual: Type, },

    // Expected type argument differs from actual type argument
    ArgTypeMismatch {expected:Type, actual:Type},

    // Attempt to apply (invoke) a term that does not have the Arrow type
    NotAFunction{actual: Type},

    // Type of the if condition different from Bool
    IfConditionMismatch { actual: Type, },

    // The two branches of an if-then-else statement have incompatible types
    BranchMismatch { then_type: Type, else_type: Type, },

    // The body of a function does not match its declared return type
    LetFunReturnTypeMismatch { expected: Type, actual: Type },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::UndefinedVariable(name) => {
                write!(f, "Undefined variable '{name}'")
            }
            TypeError::BinOpTypeMismatch {op, position, expected, actual } => {
                write!(f, "Type mismatch on the {position} side of {op}: expected {expected}, found {actual}")
            }
            TypeError::InvalidNotOperand {actual } => {
                write!(f, "Type mismatch in unary operation '~': expected Bool, found {actual}")
            }
            TypeError::ArgTypeMismatch {expected, actual} => {
                write!(f, "Invalid argument type in function call: expected {expected}, found {actual}")
            }
            TypeError::NotAFunction { actual } => {
                write!(f, "Expected function type, found {actual}")
            }
            TypeError::IfConditionMismatch { actual } => {
                write!(f, "'if' condition must be of type bool, found {actual}")
            }
            TypeError::BranchMismatch { then_type, else_type } => {
                write!(f, "Branches have incompatible types: 'then' has {then_type}, 'else' has {else_type}")
            }
            TypeError::LetFunReturnTypeMismatch { expected, actual } => {
                write!(f, "'letfun' body type does not match declared return type: expected {expected}, found {actual}")
            }
        }
    }
}

impl std::error::Error for TypeError {}

/// Typechecks a variable access (`name`) by looking up its type in the current
/// typing context, returning `TypeError::UndefinedVariable` if not found.
pub fn var_typecheck(name: &str, context: &Context) -> Result<Type, TypeError> {
    match context.lookup(name) {
        Some(typ) => Ok(typ.clone()),
        None => Err(TypeError::UndefinedVariable(name.to_owned()))
    }
}

/// Typechecks a binary operation (`first op second`) by checking both operands
/// and verifying their types against the expected operand and result types for `op`.
pub fn binop_typecheck(op: BinOp, first: &Term, second: &Term, context: &Context) -> Result<Type, TypeError> {
    // Recursive typecheck on operands
    let a = typecheck(first, context)?;
    let b = typecheck(second, context)?;
    match op{
        BinOp::Add | BinOp::Sub | BinOp::Mul => {
            if a != Type::Int {
                return Err(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Left, expected: Type::Int, actual: a.clone(), });
            }
            if b != Type::Int {
                return Err(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Right, expected: Type::Int, actual: b.clone(), });
            }
            Ok(Type::Int)
        }
        BinOp::Lt => {
            if a != Type::Int {
                return Err(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Left, expected: Type::Int, actual: a.clone(), });
            }
            if b != Type::Int {
                return Err(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Right, expected: Type::Int, actual: b.clone(), });
            }
            Ok(Type::Bool)
        }
        BinOp::And => {
            if a != Type::Bool {
                return Err(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Left, expected: Type::Bool, actual: a.clone(), });
            }
            if b != Type::Bool {
                return Err(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Right, expected: Type::Bool, actual: b.clone(), });
            }
            return Ok(Type::Bool);
        }
    }
}

/// Typechecks a logical NOT expression (`not val`) by verifying that
/// the operand evaluates to a boolean and returning `Type::Bool`.
pub fn not_typecheck (val: &Term, context: &Context) -> Result<Type, TypeError> {
    let typ = typecheck(val, context)?;
    match typ {
        Type::Bool => Ok(Type::Bool),
        _ => Err(TypeError::InvalidNotOperand {actual: typ, })
    }
}

/// Typechecks a conditional expression (`if cond then then_term else else_term`)
/// by ensuring `cond` is a boolean and both branches evaluate to identical types.
pub fn if_typecheck (cond: &Term, then: &Term, else_t: &Term, context: &Context) -> Result<Type, TypeError> {
    let typ_cond = typecheck(cond, context)?;
    if typ_cond != Type::Bool {
        return Err(TypeError::IfConditionMismatch { actual: typ_cond });
    }
    let typ1 = typecheck(then, context)?;
    let typ2 = typecheck(else_t, context)?;
    if typ1 != typ2 {
        return Err(TypeError::BranchMismatch { then_type: typ1, else_type: typ2 })
    }
    Ok(typ1)
}

/// Typechecks a function abstraction (`fun param : param_typ => body`) by checking
/// `body` in an context extended with the parameter and returning an arrow type.
pub fn fun_typecheck (param: &str, param_typ: &Type, body:&Term, context: &Context) -> Result<Type, TypeError> {
    // context extension
    let extended_env = context.extend(param.to_owned(), param_typ.clone());
    // Check the body in the extended context
    let body_type = typecheck(body, &extended_env)?;

    Ok(Type::Arrow( Box::new(param_typ.clone()), Box::new(body_type), ))
}

/// Typechecks a function application (`function argument`) by verifying that
/// `function` evaluates to an arrow type and that `argument` matches its expected parameter type.
pub fn app_typecheck (function: &Term, argument: &Term, context: &Context) -> Result<Type, TypeError> {
    let typ_arg = typecheck(argument, context)?;
    let typ_fun = typecheck(function, context)?;

    match typ_fun {
        Type::Arrow(param_type, return_type) => {
            // compare the type of the argument with the type of the parameter
            if typ_arg != *param_type {
                return Err(TypeError::ArgTypeMismatch {expected: *param_type, actual: typ_arg, });
            }
            Ok(*return_type)
        }
        _ => Err(TypeError::NotAFunction { actual: typ_fun }),
    }
}

/// Typechecks a variable binding (`let x = expr in in_term`) by inferring the
/// type of `expr` and checking `in_term` within the extended context.
pub fn let_typecheck(var_name: &str, expr: &Term, in_term: &Term, context: &Context) -> Result<Type, TypeError> {
    // get the type of the expression bounded to the variable, i.e. the type of the variable
    let expr_typ = typecheck(expr, context)?;
    // build the extended context in which the in_term will be typechecked
    let extended_env = context.extend(var_name.to_owned(), expr_typ.clone());
    // in_term typecheck in the extended_env
    typecheck(in_term, &extended_env)
}

/// Typechecks a recursive function binding (`letfun`). Verifies the arrow annotation,
/// checks that the body matches the declared return type in an context extended
/// with both function and parameter, then typechecks `in_term` in an context extended
/// only with function.
pub fn letfun_typecheck(fun_name: &str, param: &str, fun_type: &Type, body: &Term, in_term: &Term, context: &Context) -> Result<Type, TypeError>{
    match fun_type {
        Type::Arrow(param_type, return_type) => {
            // Building the enviroment with f: fun_type and param: parm_type
            let local_env = context.extend(fun_name.to_owned(), fun_type.clone())
                .extend(param.to_owned(), (**param_type).clone());
            // Check the body of the function
            let actual_body_type = typecheck(body, &local_env)?;
            if actual_body_type != **return_type {
                return Err(TypeError::LetFunReturnTypeMismatch {expected: (**return_type).clone(), actual: actual_body_type});
            }
            // context for in_term
            let in_env = context.extend(fun_name.to_owned(), fun_type.clone());
            typecheck(in_term, &in_env)
        }
        _ => Err(TypeError::NotAFunction { actual: fun_type.clone() }),
    }
}

pub fn typecheck(input: &Term, context: &Context) -> Result<Type, TypeError> {
    match input {
        // Base literals
        Term::Num(_) => Ok(Type::Int),
        Term::True | Term::False => Ok(Type::Bool),
        // Variable lookup
        Term::Var(name) => var_typecheck(name, context),
        // Operators
        Term::Not(val) => not_typecheck(val, context),
        Term::BinOp(op, a, b) => binop_typecheck(*op, a, b, context),
        // Control flow
        Term::If(cond, t1, t2) => if_typecheck(cond, t1, t2, context),
        // Bindings and functions
        Term::Let(var, expr, in_term) => let_typecheck(var, expr, in_term, context),
        Term::Fun(param, param_typ, body) => fun_typecheck(param, param_typ, body, context),
        Term::App(function, argument) => app_typecheck(function, argument, context),
        Term::LetFun(fun_name, param, fun_type, body, in_term) => letfun_typecheck(fun_name, param, fun_type, body, in_term, context)
    }
}