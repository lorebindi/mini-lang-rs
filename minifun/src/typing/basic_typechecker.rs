//! Basic monomorphic typechecker for the 'minifun' language.
//!
//! This module implements simple, syntax-directed type checking with support for:
//! - Mandatory explicit type annotations on lambda abstractions ('fun') and recursive bindings ('letfun').
//! - Scoped environment tracking for monomorphic type bindings.
//! - Static verification of base literals, unary/binary operators, conditionals, and function applications.
//! - Direct detection and reporting of type mismatches and missing annotations without unification.

use crate::ast::*;
use crate::scope::Scope;
use crate::typing::{Typechecker};
use super::types::{Type};
use super::errors::*;

pub type Context = Scope<Type>;

pub struct BasicTypechecker;

impl Typechecker for BasicTypechecker {
    fn typecheck(&mut self, term: &Term) -> Result<Type, TypecheckError> {
        let initial_context = Context::new();
        self.typecheck_term(term, &initial_context)
    }
}

impl BasicTypechecker {

    fn typecheck_term(&mut self, input: &Term, context: &Context) -> Result<Type, TypecheckError> {
        match input {
            // Base literals
            Term::Num(_) => Ok(Type::Int),
            Term::True | Term::False => Ok(Type::Bool),
            // Variable lookup
            Term::Var(name) => self.var_typecheck(name, context),
            // Operators
            Term::Not(val) => self.not_typecheck(val, context),
            Term::BinOp(op, a, b) => self.binop_typecheck(*op, a, b, context),
            // Control flow
            Term::If(cond, t1, t2) => self.if_typecheck(cond, t1, t2, context),
            // Bindings and functions
            Term::Let(var, expr, in_term) => self.let_typecheck(var, expr, in_term, context),
            Term::Fun(param, Some(param_typ), body) => self.fun_typecheck(param, param_typ, body, context),
            Term::Fun(param, None, body) => Err(TypecheckError::Annotation(AnnotationError::Missing {param: param.clone()})),
            Term::App(function, argument) => self.app_typecheck(function, argument, context),
            Term::LetFun(fun_name, param, Some(fun_type), body, in_term) => self.letfun_typecheck(fun_name, param, fun_type, body, in_term, context),
            Term::LetFun(fun_name, param, None, body, in_term) => Err(TypecheckError::Annotation(AnnotationError::Missing {param: fun_name.clone()})),
        }
    }

    /// Typechecks a variable access ('name') by looking up its type in the current
    /// typing context, returning 'TypeError::UndefinedVariable' if not found.
    fn var_typecheck(&mut self, name: &str, context: &Context) -> Result<Type, TypecheckError> {
        match context.lookup(name) {
            Some(typ) => Ok(typ.clone()),
            None => Err(TypecheckError::Type(TypeError::UndefinedVariable(name.to_owned())))
        }
    }

    /// Typechecks a logical NOT expression (`not val`) by verifying that
    /// the operand evaluates to a boolean and returning `Type::Bool`.
    pub fn not_typecheck (&mut self, val: &Term, context: &Context) -> Result<Type, TypecheckError> {
        let typ = self.typecheck_term(val, context)?;
        match typ {
            Type::Bool => Ok(Type::Bool),
            _ => Err(TypecheckError::Type(TypeError::InvalidNotOperand {actual: typ, }))
        }
    }

    /// Typechecks a binary operation ('first op second') by checking both operands
    /// and verifying their types against the expected operand and result types for 'op'.
    fn binop_typecheck(&mut self, op: BinOp, first: &Term, second: &Term, context: &Context) -> Result<Type, TypecheckError> {
        // Recursive typecheck on operands
        let a = self.typecheck_term(first, context)?;
        let b = self.typecheck_term(second, context)?;
        match op{
            BinOp::Add | BinOp::Sub | BinOp::Mul => {
                if a != Type::Int {
                    return Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Left, expected: Type::Int, actual: a.clone(), }));
                }
                if b != Type::Int {
                    return Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Right, expected: Type::Int, actual: b.clone(), }));
                }
                Ok(Type::Int)
            }
            BinOp::Lt => {
                if a != Type::Int {
                    return Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Left, expected: Type::Int, actual: a.clone(), }));
                }
                if b != Type::Int {
                    return Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Right, expected: Type::Int, actual: b.clone(), }));
                }
                Ok(Type::Bool)
            }
            BinOp::And => {
                if a != Type::Bool {
                    return Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Left, expected: Type::Bool, actual: a.clone(), }));
                }
                if b != Type::Bool {
                    return Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Right, expected: Type::Bool, actual: b.clone(), }));
                }
                return Ok(Type::Bool);
            }
        }
    }

    /// Typechecks a conditional expression ('if cond then then_term else else_term')
    /// by ensuring 'cond' is a boolean and both branches evaluate to identical types.
    fn if_typecheck (&mut self, cond: &Term, then: &Term, else_t: &Term, context: &Context) -> Result<Type, TypecheckError> {
        let typ_cond = self.typecheck_term(cond, context)?;
        if typ_cond != Type::Bool {
            return Err(TypecheckError::Type(TypeError::IfConditionMismatch { actual: typ_cond }));
        }
        let typ1 = self.typecheck_term(then, context)?;
        let typ2 = self.typecheck_term(else_t, context)?;
        if typ1 != typ2 {
            return Err(TypecheckError::Type(TypeError::BranchMismatch { then_type: typ1, else_type: typ2 }))
        }
        Ok(typ1)
    }

    /// Typechecks a function abstraction ('fun param : param_typ => body') by checking
    /// 'body' in an context extended with the parameter and returning an arrow type.
    fn fun_typecheck (&mut self, param: &str, param_typ: &Type, body:&Term, context: &Context) -> Result<Type, TypecheckError> {
        // context extension
        let extended_env = context.extend(param.to_owned(), param_typ.clone());
        // Check the body in the extended context
        let body_type = self.typecheck_term(body, &extended_env)?;

        Ok(Type::Arrow( Box::new(param_typ.clone()), Box::new(body_type), ))
    }

    /// Typechecks a function application ('function argument') by verifying that
    /// 'function' evaluates to an arrow type and that 'argument' matches its expected parameter type.
    fn app_typecheck (&mut self, function: &Term, argument: &Term, context: &Context) -> Result<Type, TypecheckError> {
        let typ_arg = self.typecheck_term(argument, context)?;
        let typ_fun = self.typecheck_term(function, context)?;

        match typ_fun {
            Type::Arrow(param_type, return_type) => {
                // compare the type of the argument with the type of the parameter
                if typ_arg != *param_type {
                    return Err(TypecheckError::Type(TypeError::ArgTypeMismatch {expected: *param_type, actual: typ_arg, }));
                }
                Ok(*return_type)
            }
            _ => Err(TypecheckError::Type(TypeError::NotAFunction { actual: typ_fun })),
        }
    }

    /// Typechecks a variable binding ('let x = expr in in_term') by inferring the
    /// type of 'expr' and checking 'in_term' within the extended context.
    fn let_typecheck(&mut self, var_name: &str, expr: &Term, in_term: &Term, context: &Context) -> Result<Type, TypecheckError> {
        // get the type of the expression bounded to the variable, i.e. the type of the variable
        let expr_typ = self.typecheck_term(expr, context)?;
        // build the extended context in which the in_term will be typechecked
        let extended_env = context.extend(var_name.to_owned(), expr_typ.clone());
        // in_term typecheck in the extended_env
        self.typecheck_term(in_term, &extended_env)
    }

    /// Typechecks a recursive function binding ('letfun'). Verifies the arrow annotation,
    /// checks that the body matches the declared return type in an context extended
    /// with both function and parameter, then typechecks 'in_term' in an context extended
    /// only with function.
    fn letfun_typecheck(&mut self, fun_name: &str, param: &str, fun_type: &Type, body: &Term, in_term: &Term, context: &Context) -> Result<Type, TypecheckError>{
        match fun_type {
            Type::Arrow(param_type, return_type) => {
                // Building the enviroment with f: fun_type and param: parm_type
                let local_env = context.extend(fun_name.to_owned(), fun_type.clone())
                    .extend(param.to_owned(), (**param_type).clone());
                // Check the body of the function
                let actual_body_type = self.typecheck_term(body, &local_env)?;
                if actual_body_type != **return_type {
                    return Err(TypecheckError::Type(TypeError::LetFunReturnTypeMismatch {expected: (**return_type).clone(), actual: actual_body_type}));
                }
                // context for in_term
                let in_env = context.extend(fun_name.to_owned(), fun_type.clone());
                self.typecheck_term(in_term, &in_env)
            }
            _ => Err(TypecheckError::Type(TypeError::NotAFunction { actual: fun_type.clone() })),
        }
    }
}


// Some unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::typing::errors::TypeError;

    // Local builders
    fn t_int() -> Type { Type::Int }
    fn t_bool() -> Type { Type::Bool }
    fn t_arr(from: Type, to: Type) -> Type { Type::Arrow(Box::new(from), Box::new(to)) }
    fn var(s: &str) -> Term { Term::Var(s.to_string()) }
    fn app(f: Term, a: Term) -> Term { Term::App(Box::new(f), Box::new(a)) }
    fn fun(p: &str, t: Type, b: Term) -> Term { Term::Fun(p.to_string(), Some(t), Box::new(b)) }

    fn assert_type_in(term: &Term, env: &Context, expected: Type) {
        let mut checker = BasicTypechecker;
        assert_eq!(checker.typecheck_term(term, env), Ok(expected));
    }

    #[test]
    fn test_var_defined() {
        let env = Context::new().extend("x".to_string(), t_int());
        assert_type_in(&var("x"), &env, t_int());
    }

    #[test]
    fn test_var_shadowing_uses_innermost_binding() {
        let env = Context::new()
            .extend("x".to_string(), t_int())
            .extend("x".to_string(), t_bool());
        assert_type_in(&var("x"), &env, t_bool());
    }

    #[test]
    fn test_fun_param_shadows_outer_binding() {
        let env = Context::new().extend("x".to_string(), t_bool());
        assert_type_in(&fun("x", t_int(), var("x")), &env, t_arr(t_int(), t_int()));
    }

    #[test]
    fn test_app_simple() {
        let env = Context::new().extend("f".to_string(), t_arr(t_int(), t_bool()));
        assert_type_in(&app(var("f"), Term::Num(1)), &env, t_bool());
    }

    #[test]
    fn test_app_left_associative_curried_calls() {
        let env = Context::new().extend("f".to_string(), t_arr(t_int(), t_arr(t_int(), t_int())));
        assert_type_in(&app(app(var("f"), Term::Num(1)), Term::Num(2)), &env, t_int());
    }

    #[test]
    fn test_app_applying_non_function() {
        let env = Context::new().extend("x".to_string(), t_int());
        let mut checker = BasicTypechecker;
        assert_eq!(
            checker.typecheck_term(&app(var("x"), Term::Num(1)), &env),
            Err(TypecheckError::Type(TypeError::NotAFunction { actual: t_int() }))
        );
    }

    #[test]
    fn test_app_argument_type_mismatch() {
        let env = Context::new().extend("f".to_string(), t_arr(t_int(), t_bool()));
        let mut checker = BasicTypechecker;
        assert_eq!(
            checker.typecheck_term(&app(var("f"), Term::True), &env),
            Err(TypecheckError::Type(TypeError::ArgTypeMismatch { expected: t_int(), actual: t_bool() }))
        );
    }
}