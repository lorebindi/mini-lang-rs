//! Error types and diagnostic reporting for the 'minifun' typechecking subsystem.
//!
//! This module defines the error taxonomy structured across three distinct categories:
//! - 'TypeError': High-level semantic violations (e.g., operator mismatches, undefined identifiers,
//!   branch incompatibility, cyclic types).
//! - 'AnnotationError': Syntactic contract failures concerning missing or unexpected type annotations.
//! - 'UnifyError': Structural mismatch and occurs-check failures produced by Robinson unification.
//!
//! All error categories converge into the unified 'TypecheckError' entrypoint.

use std::fmt;
use crate::ast::BinOp;
use super::types::{TypeVar, Type};


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypecheckError {
    /// Semantic type violations, such as operator mismatches or missing variables.
    Type(TypeError),
    /// Syntax contract violations regarding expected or unexpected type annotations.
    Annotation(AnnotationError),
    /// Low-level equation-solving failures during type unification.
    Unification(UnifyError),
}

/// --------------------------------- Type Errors ---------------------------------

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

    // Unifying `var` with `ty` would create an infinite/cyclic type (occurs check failed)
    InfiniteType { var: TypeVar, ty: Type },
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
            TypeError::InfiniteType { var, ty } => {
                write!(f, "Infinite type: type variable '{var}' occurs in {ty}, which would create a cyclic type")
            }
        }
    }
}

impl std::error::Error for TypeError {}

/// --------------------------------- Unify Error ---------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifyError {
    // Structural mismatch between incompatible types (e.g., int vs bool, or primitive vs function)
    Mismatch { expected: Type, actual: Type },
    // Occurs-check failure: binding `var` would create a cyclic/infinite type because it appears free in `ty`
    OccursCheck { var: TypeVar, ty: Type },
}

impl fmt::Display for UnifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnifyError::Mismatch { expected, actual } => {
                write!(f, "Cannot unify {expected} with {actual}")
            }
            UnifyError::OccursCheck { var, ty } => {
                write!(f, "Occurs check failed: variable {var} occurs in {ty}")
            }
        }
    }
}

impl std::error::Error for UnifyError {}

/// --------------------------------- Annotation Errors ---------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationError {
    /// The basic checker ask for the annotations.
    Missing { param: String },
    /// The HM checker doesn't accept annotations.
    Unexpected { target: String, ty: Type,  },
}

impl fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnnotationError::Missing { param } => {
                write!(f, "missing type annotation for parameter '{param}'")
            }
            AnnotationError::Unexpected { target, ty } => {
                write!(f, "unexpected type annotation '{ty}' on '{target}'")
            }
        }
    }
}

impl std::error::Error for AnnotationError {}