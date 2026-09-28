use crate::ast::*;
use super::types::{Context, TypeVar, Type, TypeSchema};
use super::errors::*;
use super::substitution::*;

/// Hindley-Milner type checker
pub struct HmTypeChecker {
    fresh_gen: FreshVarGen,
    subst: Substitution,
}

impl HmTypeChecker {
    pub fn new() -> Self {
        Self { fresh_gen: FreshVarGen::new(), subst: Substitution::new() }
    }

    pub fn typecheck(&mut self, input: &Term, context: &Context) -> Result<Type, TypeError> {
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
            Term::Let(var, expr, in_term) => let_typecheck(var, expr, in_term, context),
            Term::Fun(param, body) => self.fun_typecheck(param, body, context),
            Term::App(function, argument) => app_typecheck(function, argument, context),
            Term::LetFun(fun_name, param, body, in_term) => letfun_typecheck(fun_name, param, body, in_term, context)
        }
    }

    /// Typechecks a variable access (`name`) by looking up its type in the current
    /// typing context, returning `TypeError::UndefinedVariable` if not found.
    fn var_typecheck(&mut self, name: &str, context: &Context) -> Result<Type, TypeError> {
        match context.lookup(name) {
            Some(type_schema) => Ok(type_schema.inst(&mut self.fresh_gen)),
            None => Err(TypeError::UndefinedVariable(name.to_owned()))
        }
    }

    /// Typechecks a logical NOT expression (`not val`) by verifying that
    /// the operand evaluates to a boolean and returning `Type::Bool`.
    fn not_typecheck (&mut self, val: &Term, context: &Context) -> Result<Type, TypeError> {
        let typ = self.typecheck(val, context)?;

        // Apply the current substitution before unification (to resolve any variables that are already constrained)
        let typ_resolved = self.subst.apply_to_type(&typ);

        match typ_resolved.unify(&Type::Bool) {
            Ok(s) => {
                self.subst.compose(&s);
                Ok(Type::Bool)
            }
            Err(_) => Err(TypeError::InvalidNotOperand {actual: typ_resolved, })
        }
    }

    /// Typechecks a binary operation (`first op second`) by checking both operands
    /// and verifying their types against the expected operand and result types for `op`.
    fn binop_typecheck(&mut self, op: BinOp, first: &Term, second: &Term, context: &Context) -> Result<Type, TypeError> {

        // Determines the expected type of the operands and the return type based on the operator
        let (expected_operand_ty, result_ty) = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul => (Type::Int, Type::Int),
            BinOp::Lt => (Type::Int, Type::Bool),
            BinOp::And => (Type::Bool, Type::Bool),
        };

        // Recursive typecheck on the first operand
        let a = self.typecheck(first, context)?;
        let a_resolved = self.subst.apply_to_type(&a);

        // Unification of the first operand resolved type with the expected type
        match a_resolved.unify(&expected_operand_ty) {
            Ok(s1) => self.subst.compose(&s1),
            Err(_) => {
                return Err(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Left, expected: expected_operand_ty, actual: a_resolved.clone(), });
            }
        };

        // Recursive typecheck on the second operand
        let b = self.typecheck(second, context)?;
        let b_resolved = self.subst.apply_to_type(&b);

        // Unification of the second operand resolved type with the expected type
        match a_resolved.unify(&expected_operand_ty) {
            Ok(s2) => self.subst.compose(&s2),
            Err(_) => {
                return Err(TypeError::BinOpTypeMismatch {op: op, position: OperandPosition::Right, expected: expected_operand_ty, actual: b_resolved.clone(), });
            }
        };

        Ok(result_ty)
    }

    /// Typechecks a conditional expression (`if cond then then_term else else_term`)
    /// by ensuring `cond` is a boolean and both branches evaluate to identical types.
    fn if_typecheck (&mut self, cond: &Term, then: &Term, else_t: &Term, context: &Context) -> Result<Type, TypeError> {

        // Recursive typecheck on the condition
        let typ_cond = self.typecheck(cond, context)?;
        let typ_cond_resolved = self.subst.apply_to_type(&typ_cond);
        // Unification of the condition type with the expected condition type
        match typ_cond_resolved.unify(&Type::Bool) {
            Ok(s) => self.subst.compose(&s),
            Err(_) => return Err(TypeError::IfConditionMismatch { actual: typ_cond_resolved})
        };

        // Recursive typecheck branches
        let typ_then = self.typecheck(then, context)?;
        let typ_else = self.typecheck(else_t, context)?;
        let typ_then_resolved = self.subst.apply_to_type(&typ_then);
        let typ_else_resolved = self.subst.apply_to_type(&typ_else);

        // Unify the branches
        match typ_then_resolved.unify(&typ_else_resolved) {
            Ok(s) => {
                self.subst.compose(&s);
                // The resulting type is the common type passed through the last substitution
                Ok(self.subst.apply_to_type(&typ_then_resolved))
            }
            Err(_) => Err(TypeError::BranchMismatch { then_type: typ_then_resolved, else_type: typ_else_resolved })
        }

    }

    /// Typechecks a function abstraction ('fun param : param_typ => body') by checking
    /// 'body' in an context extended with the parameter and returning an arrow type.
    pub fn fun_typecheck (&mut self, param: &str, body:&Term, context: &Context) -> Result<Type, TypeError> {

        // Create new type variable for the parameter of the function
        let param_ty = Type::Var(self.fresh_gen.fresh());
        // Extend the current context with the association between the param name and param type
        let extended_context = context.extend(param.parse().unwrap(), TypeSchema::new_monomorphic(param_ty.clone()));

        // Check the body in the extended context
        let body_ty = self.typecheck(body, &extended_context)?;

        let param_ty_resolved = self.subst.apply_to_type(&param_ty);
        let body_ty_resolved = self.subst.apply_to_type(&body_ty);

        Ok(Type::Arrow( Box::new(param_ty_resolved.clone()), Box::new(body_ty_resolved), ))
    }
}


/*

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
}*/