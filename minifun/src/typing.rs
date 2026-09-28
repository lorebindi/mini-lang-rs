//! Type checking and inference subsystem for the 'minifun' language.
//!
//! This module coordinates the typing architecture and exports:
//! - Core type definitions, type schemas, and type variable representations.
//! - The unified 'Typechecker' trait implemented by both the basic monomorphic checker
//!   and the Hindley-Milner inference engine.
//! - Diagnostic error hierarchies separating semantic type errors, annotation contract violations,
//!   and unification failures.
//! - Environment scope extensions for tracking free type variables across nested lexical frames.

pub mod errors;
pub mod types;
pub mod substitution;
pub mod hindley_milner_typechecker;
pub mod basic_typechecker;

use std::collections::HashSet;
pub use errors::TypecheckError;
pub use types::{TypeVar, Type, TypeSchema};
pub use substitution::Substitution;
pub use hindley_milner_typechecker::HmTypeChecker;

use crate::ast::Term;
use crate::scope::Scope;

// Common trait for all the typecheckers
pub trait Typechecker {
    fn typecheck(&mut self, input: &Term) -> Result<Type, TypecheckError> ;
}

impl Scope<TypeSchema> {
    /// Collects all free type variables present in the environment and its enclosing scopes.
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        let mut vars: HashSet<TypeVar> = self.bindings
            .values()
            .flat_map(|schema| schema.free_vars())
            .collect();

        if let Some(parent) = &self.parent {
            vars.extend(parent.free_vars());
        }

        vars
    }
}