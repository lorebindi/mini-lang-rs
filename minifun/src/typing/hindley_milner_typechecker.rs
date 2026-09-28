//! Hindley-Milner type inference engine (Algorithm W) for the 'minifun' language.
//!
//! This module implements constraint-based type reconstruction with support for:
//! - Unannotated function abstractions and parametric polymorphism via let-bound generalization.
//! - Fresh type variable generation and cumulative substitution composition.
//! - Rejection of explicit type annotations to enforce pure type inference.

use crate::ast::*;
use crate::typing::Typechecker;
use crate::scope::*;
use super::types::{Type, TypeSchema};
use super::errors::*;
use super::substitution::*;

pub type Context = Scope<TypeSchema>;

/// Hindley-Milner type checker
pub struct HmTypeChecker {
    fresh_gen: FreshVarGen,
    subst: Substitution,
}

impl Typechecker for HmTypeChecker {
    fn typecheck(&mut self, term: &Term) -> Result<Type, TypecheckError> {
        // 1. Istanzia un contesto vuoto
        let initial_context = Context::new();

        // 2. Esegue l'inferenza ricorsiva
        let raw_type = self.typecheck_term(term, &initial_context)?;

        // 3. Risolve eventuali variabili residue con le unificazioni accumulate
        Ok(self.subst.apply_to_type(&raw_type))
    }
}

impl HmTypeChecker {
    pub fn new() -> Self {
        Self { fresh_gen: FreshVarGen::new(), subst: Substitution::new() }
    }

    /// Unifies two types and, if successful, stores the result in 'self.subst'.
    /// Returns the raw unification error; the caller decides how to handle it.
    fn unify_and_compose(&mut self, t1: &Type, t2: &Type) -> Result<(), UnifyError> {
        let s = t1.unify(t2)?;
        self.subst = self.subst.compose(&s);
        Ok(())
    }

    pub fn typecheck_term(&mut self, input: &Term, context: &Context) -> Result<Type, TypecheckError> {
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
            Term::Fun(param, None, body) => self.fun_typecheck(param, body, context),
            Term::Fun(param, Some(t), body) =>
                Err(TypecheckError::Annotation(AnnotationError::Unexpected {target: param.clone(), ty: t.clone(), })),
            Term::App(function, argument) => self.app_typecheck(function, argument, context),
            Term::LetFun(fun_name, param, None, body, in_term) => self.letfun_typecheck(fun_name, param, body, in_term, context),
            Term::LetFun(fun_name, param, Some(t), body, in_term) =>
                Err(TypecheckError::Annotation(AnnotationError::Unexpected {target: fun_name.clone(), ty: t.clone(),})),
        }
    }

    /// Typechecks a variable access (`name`) by looking up its type in the current
    /// typing context, returning `TypeError::UndefinedVariable` if not found.
    fn var_typecheck(&mut self, name: &str, context: &Context) -> Result<Type, TypecheckError> {
        match context.lookup(name) {
            Some(type_schema) => Ok(type_schema.inst(&mut self.fresh_gen)),
            None => Err(TypecheckError::Type(TypeError::UndefinedVariable(name.to_owned())))
        }
    }

    /// Typechecks a logical NOT expression (`not val`) by verifying that
    /// the operand evaluates to a boolean and returning `Type::Bool`.
    fn not_typecheck (&mut self, val: &Term, context: &Context) -> Result<Type, TypecheckError> {
        let typ = self.typecheck_term(val, context)?;

        // Apply the current substitution before unification (to resolve any variables that are already constrained)
        let typ_resolved = self.subst.apply_to_type(&typ);

        match self.unify_and_compose(&typ_resolved, &Type::Bool) {
            Ok(()) => Ok(Type::Bool),
            Err(_) => Err(TypecheckError::Type(TypeError::InvalidNotOperand {actual: typ_resolved, }))
        }
    }

    /// Typechecks a binary operation (`first op second`) by checking both operands
    /// and verifying their types against the expected operand and result types for `op`.
    fn binop_typecheck(&mut self, op: BinOp, first: &Term, second: &Term, context: &Context) -> Result<Type, TypecheckError> {

        // Determines the expected type of the operands and the return type based on the operator
        let (expected_operand_ty, result_ty) = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul => (Type::Int, Type::Int),
            BinOp::Lt => (Type::Int, Type::Bool),
            BinOp::And => (Type::Bool, Type::Bool),
        };

        // Recursive typecheck on the first operand
        let a = self.typecheck_term(first, context)?;
        let a_resolved = self.subst.apply_to_type(&a);

        // Unification of the first operand resolved type with the expected type
        if self.unify_and_compose(&a_resolved, &expected_operand_ty).is_err() {
            // occurs check cannot be launched
            return Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {
                op, position: OperandPosition::Left, expected: expected_operand_ty, actual: a_resolved,
            }));
        }

        // Recursive typecheck on the second operand
        let b = self.typecheck_term(second, context)?;
        let b_resolved = self.subst.apply_to_type(&b);

        // Unification of the second operand resolved type with the expected type
        if self.unify_and_compose(&b_resolved, &expected_operand_ty).is_err() {
            // occurs check cannot be launched
            return Err(TypecheckError::Type(TypeError::BinOpTypeMismatch {
                op, position: OperandPosition::Right, expected: expected_operand_ty, actual: b_resolved,
            }));
        }

        Ok(result_ty)
    }

    /// Typechecks a conditional expression (`if cond then then_term else else_term`)
    /// by ensuring `cond` is a boolean and both branches evaluate to identical types.
    fn if_typecheck (&mut self, cond: &Term, then: &Term, else_t: &Term, context: &Context) -> Result<Type, TypecheckError> {

        // Recursive typecheck on the condition
        let typ_cond = self.typecheck_term(cond, context)?;
        let typ_cond_resolved = self.subst.apply_to_type(&typ_cond);
        // Unification of the condition type with the expected condition type
        if self.unify_and_compose(&typ_cond_resolved, &Type::Bool).is_err() {
            // occur check cannot happen against a Type::Bool
            return Err(TypecheckError::Type(TypeError::IfConditionMismatch { actual: typ_cond_resolved }));
        }

        // Recursive typecheck branches
        let typ_then = self.typecheck_term(then, context)?;
        let typ_else = self.typecheck_term(else_t, context)?;
        let typ_then_resolved = self.subst.apply_to_type(&typ_then);
        let typ_else_resolved = self.subst.apply_to_type(&typ_else);

        // Unify the branches
        match self.unify_and_compose(&typ_then_resolved, &typ_else_resolved) {
            Ok(()) => Ok(self.subst.apply_to_type(&typ_then_resolved)),
            Err(UnifyError::Mismatch { .. }) => Err(TypecheckError::Type(TypeError::BranchMismatch {
                then_type: typ_then_resolved,
                else_type: typ_else_resolved,
            })),
            Err(UnifyError::OccursCheck { var, ty }) => Err(TypecheckError::Type(TypeError::InfiniteType { var, ty })),
        }
    }

    /// Typechecks a function abstraction ('fun param : param_typ => body') by checking
    /// 'body' in an context extended with the parameter and returning an arrow type.
    pub fn fun_typecheck (&mut self, param: &str, body:&Term, context: &Context) -> Result<Type, TypecheckError> {

        // Create new type variable for the parameter of the function
        let param_ty = Type::Var(self.fresh_gen.fresh());
        // Extend the current context with the association between the param name and param type
        let extended_context = context.extend(param.to_owned(), TypeSchema::new_monomorphic(param_ty.clone()));

        // Check the body in the extended context
        let body_ty = self.typecheck_term(body, &extended_context)?;

        let param_ty_resolved = self.subst.apply_to_type(&param_ty);
        let body_ty_resolved = self.subst.apply_to_type(&body_ty);

        Ok(Type::Arrow( Box::new(param_ty_resolved.clone()), Box::new(body_ty_resolved), ))
    }

    /// Typechecks a variable binding ('let x = expr in in_term') by inferring the
    /// type of 'expr' and checking 'in_term' within the extended context.
    fn let_typecheck(&mut self, var_name: &str, expr: &Term, in_term: &Term, context: &Context) -> Result<Type, TypecheckError> {
        // get the type of the expression bounded to the variable, i.e. the type of the variable
        let expr_typ = self.typecheck_term(expr, context)?;
        let expr_typ_resolved = self.subst.apply_to_type(&expr_typ);
        let schema = expr_typ_resolved.generalize(&context.free_vars()); // quantify over vars free in expr_typ but not in context
        // build the extended context in which the in_term will be typechecked
        let extended_env = context.extend(var_name.to_owned(), schema);
        // in_term typecheck in the extended_env
        self.typecheck_term(in_term, &extended_env)
    }

    /// Typechecks a function application ('function argument') by verifying that
    /// 'function' evaluates to an arrow type and that 'argument' matches its expected parameter type.
    fn app_typecheck(&mut self, function: &Term, argument: &Term, context: &Context) -> Result<Type, TypecheckError> {
        let typ_arg = self.typecheck_term(argument, context)?;
        let typ_arg_resolved = self.subst.apply_to_type(&typ_arg);
        let typ_fun = self.typecheck_term(function, context)?;
        let typ_fun_resolved = self.subst.apply_to_type(&typ_fun);

        // typ_fun must unify with "typ_arg -> ?", where ? is a fresh variable
        // standing for the (currently unknown) return type.
        let ret_var = Type::Var(self.fresh_gen.fresh());
        let expected_fun_type = Type::Arrow(Box::new(typ_arg_resolved.clone()), Box::new(ret_var.clone()));

        match self.unify_and_compose(&typ_fun_resolved, &expected_fun_type) {
            Ok(()) => Ok(self.subst.apply_to_type(&ret_var)),
            Err(UnifyError::Mismatch { .. }) => match &typ_fun_resolved {
                Type::Arrow(param_type, _) => Err(TypecheckError::Type(TypeError::ArgTypeMismatch {
                    expected: (**param_type).clone(),
                    actual: typ_arg_resolved,
                })),
                _ => Err(TypecheckError::Type(TypeError::NotAFunction { actual: typ_fun_resolved })),
            },
            Err(UnifyError::OccursCheck { var, ty }) => Err(TypecheckError::Type(TypeError::InfiniteType { var, ty })),
        }
    }

    fn letfun_typecheck(&mut self, fun_name: &str, param: &str, body: &Term, in_term: &Term, context: &Context) -> Result<Type, TypecheckError> {
        // Introduce fresh variables for the recursive function and its argument
        let rec_fun_var = Type::Var(self.fresh_gen.fresh());
        let arg_var = Type::Var(self.fresh_gen.fresh());
        // Check the body of the function with the extended context
        let extended_ctx = context.extend(fun_name.to_owned(), TypeSchema::new_monomorphic(rec_fun_var.clone()))
            .extend(param.to_owned(),  TypeSchema::new_monomorphic(arg_var.clone()));
        let typ_body = self.typecheck_term(body, &extended_ctx)?;
        let typ_body_resolved = self.subst.apply_to_type(&typ_body);
        // Try to unify the variable for f with its expected function type
        let arg_var_resolved = self.subst.apply_to_type(&arg_var);
        let rec_fun_var_resolved = self.subst.apply_to_type(&rec_fun_var);
        let expected_fun_typ = Type::Arrow(Box::new(arg_var_resolved), Box::new(typ_body_resolved));
        match self.unify_and_compose(&rec_fun_var_resolved, &expected_fun_typ) {
            Ok(()) => {}
            Err(UnifyError::Mismatch { expected, actual }) => {
                return Err(TypecheckError::Type(TypeError::LetFunReturnTypeMismatch { expected, actual }));
            }
            Err(UnifyError::OccursCheck { var, ty }) => {
                return Err(TypecheckError::Type(TypeError::InfiniteType { var, ty }));
            }
        }

        let fun_typ = self.subst.apply_to_type(&expected_fun_typ);

        let schema = fun_typ.generalize(&context.free_vars());
        let in_env = context.extend(fun_name.to_owned(), schema);
        self.typecheck_term(in_term, &in_env)
    }
}
