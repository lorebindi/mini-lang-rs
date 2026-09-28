//! Type substitutions and fresh variable generation for Hindley-Milner type inference.
//!
//! This module provides the core stateful machinery required by Algorithm W:
//! - 'FreshVarGen': A monotonic counter generator for distinct type variables.
//! - 'Substitution': A finite mapping from type variables to concrete monotypes.
//! - Transitive resolution of monotypes and variable-capture-free scheme specialization.
//! - Substitution composition implementing the associative algebraic property \((s_1 \circ s_2)(T) = s_1(s_2(T))\).


use std::collections::{HashMap, HashSet};
use super::types::{Type, TypeSchema, TypeVar};

/// Fresh type variable generator
pub struct FreshVarGen {
    next: TypeVar,
}

impl FreshVarGen {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    pub fn fresh(&mut self) -> TypeVar {
        let v = self.next;
        self.next += 1;
        v
    }
}

/// Represents a finite mapping from type variables to types (S: TypeVar -> Type).
///
/// Used during type inference to record unification constraints and specialize
/// types by replacing type variables with resolved monotypes.
pub struct Substitution {
    mapping: HashMap<TypeVar, Type>,
}

impl Substitution {

    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }

    /// Assign a concrete 'Type' to a 'TypeVar'
    pub fn insert(&mut self, var: TypeVar, ty: Type) {
        self.mapping.insert(var, ty);
    }

    /// Applies the current substitution to a monotype, transitively resolving all type variables.
    pub fn apply_to_type(&self, ty: &Type) -> Type {
        self.apply_to_type_excluding(ty, &HashSet::new())
    }

    /// Recursively applies the substitution to a type, skipping any universally quantified
    /// (quantified_vars) variables to prevent variable capture.
    fn apply_to_type_excluding(&self, ty: &Type, quantified_vars: &HashSet<TypeVar>) -> Type {
        match ty {
            Type::Int => Type::Int,
            Type::Bool => Type::Bool,
            Type::Arrow(t1, t2) => {
                Type::Arrow(Box::new(self.apply_to_type_excluding(t1, quantified_vars)), Box::new(self.apply_to_type_excluding(t2, quantified_vars)))
            }
            Type::Var(var_id) => {
                if quantified_vars.contains(var_id) {
                    ty.clone()
                } else if let Some(curr_ty) = self.mapping.get(var_id) {
                    self.apply_to_type_excluding(curr_ty, quantified_vars)
                } else {
                    ty.clone()
                }
            }
        }
    }

    /// Applies the substitution to a polytype (type scheme), replacing only its free type variables
    /// while leaving quantified variables untouched.
    pub fn apply_to_schema(&self, schema: &TypeSchema) -> TypeSchema {
        let quantified_vars: HashSet<TypeVar> = schema.quantified_vars.iter().copied().collect();
        TypeSchema {
            quantified_vars: schema.quantified_vars.clone(),
            ty: self.apply_to_type_excluding(&schema.ty, &quantified_vars),
        }
    }

    /// Composes this substitution ('self') with another substitution ('other'),
    /// producing a new substitution equivalent to '(self \composition other)'.
    ///
    /// Applying the resulting substitution to a type 'T' is equivalent to:
    /// 'self.apply_to_type(other.apply_to_type(T))'.
    pub fn compose (&self, other: &Substitution) -> Substitution {
        let mut result = HashMap::new();

        // Apply 'self' to all type targets in 'other'
        for (&var, ty) in &other.mapping {
            result.insert(var, self.apply_to_type(ty));
        }

        // Add bindings from 'self' that are not already present in 'other'
        for (&var, ty) in &self.mapping {
            result.entry(var).or_insert_with(|| ty.clone());
        }

        Substitution { mapping: result }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn var(n: TypeVar) -> Type { Type::Var(n) }
    fn arrow(a: Type, b: Type) -> Type { Type::Arrow(Box::new(a), Box::new(b)) }

    fn sub(pairs: &[(TypeVar, Type)]) -> Substitution {
        let mut s = Substitution::new();
        for (v, t) in pairs {
            s.insert(*v, t.clone());
        }
        s
    }

    // FreshVarGen

    #[test]
    fn fresh_vars_are_distinct() {
        let mut g = FreshVarGen::new();
        let a = g.fresh();
        let b = g.fresh();
        let c = g.fresh();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    // apply_to_type

    #[test]
    fn empty_substitution_is_identity() {
        let t = arrow(var(0), arrow(Type::Int, var(1)));
        assert_eq!(Substitution::new().apply_to_type(&t), t);
    }

    #[test]
    fn apply_replaces_bound_var() {
        let s = sub(&[(0, Type::Int)]);
        assert_eq!(s.apply_to_type(&var(0)), Type::Int);
    }

    #[test]
    fn apply_leaves_unbound_var() {
        let s = sub(&[(0, Type::Int)]);
        assert_eq!(s.apply_to_type(&var(1)), var(1));
    }

    #[test]
    fn apply_descends_into_arrows() {
        let s = sub(&[(0, Type::Int), (1, Type::Bool)]);
        let t = arrow(var(0), arrow(var(1), var(2)));
        assert_eq!(
            s.apply_to_type(&t),
            arrow(Type::Int, arrow(Type::Bool, var(2)))
        );
    }

    #[test]
    fn apply_is_transitive() {
        // a -> b, b -> Int  ==>  a resolves to Int
        let s = sub(&[(0, var(1)), (1, Type::Int)]);
        assert_eq!(s.apply_to_type(&var(0)), Type::Int);
    }

    #[test]
    fn apply_is_idempotent_on_resolved_types() {
        let s = sub(&[(0, arrow(Type::Int, var(1))), (1, Type::Bool)]);
        let once = s.apply_to_type(&var(0));
        let twice = s.apply_to_type(&once);
        assert_eq!(once, twice);
        assert_eq!(once, arrow(Type::Int, Type::Bool));
    }

    // apply_to_schema

    #[test]
    fn schema_quantified_vars_are_not_substituted() {
        // forall 0. 0 -> 1   with   {0 -> Int, 1 -> Bool}
        let schema = TypeSchema {
            quantified_vars: vec![0],
            ty: arrow(var(0), var(1)),
        };
        let s = sub(&[(0, Type::Int), (1, Type::Bool)]);
        let result = s.apply_to_schema(&schema);
        assert_eq!(result.quantified_vars, vec![0]);
        assert_eq!(result.ty, arrow(var(0), Type::Bool));
    }

    #[test]
    fn schema_without_quantified_vars_behaves_like_type() {
        let schema = TypeSchema {
            quantified_vars: vec![],
            ty: arrow(var(0), var(0)),
        };
        let s = sub(&[(0, Type::Int)]);
        assert_eq!(
            s.apply_to_schema(&schema).ty,
            arrow(Type::Int, Type::Int)
        );
    }

    // compose

    #[test]
    fn compose_matches_documented_semantics() {
        // (s1 ∘ s2)(T) == s1(s2(T))
        let s1 = sub(&[(1, Type::Int)]);
        let s2 = sub(&[(0, arrow(var(1), var(2)))]);
        let t = arrow(var(0), var(1));

        let composed = s1.compose(&s2);
        let expected = s1.apply_to_type(&s2.apply_to_type(&t));
        assert_eq!(composed.apply_to_type(&t), expected);
        assert_eq!(expected, arrow(arrow(Type::Int, var(2)), Type::Int));
    }

    #[test]
    fn compose_other_wins_on_shared_var() {
        // other is applied first, so its binding for 0 is the one that survives
        let s1 = sub(&[(0, Type::Int)]);
        let s2 = sub(&[(0, Type::Bool)]);
        assert_eq!(s1.compose(&s2).apply_to_type(&var(0)), Type::Bool);
    }

    #[test]
    fn compose_keeps_self_bindings_absent_in_other() {
        let s1 = sub(&[(0, Type::Int)]);
        let s2 = sub(&[(1, Type::Bool)]);
        let c = s1.compose(&s2);
        assert_eq!(c.apply_to_type(&var(0)), Type::Int);
        assert_eq!(c.apply_to_type(&var(1)), Type::Bool);
    }

    #[test]
    fn compose_with_empty_is_identity() {
        let s = sub(&[(0, arrow(var(1), Type::Int))]);
        let t = arrow(var(0), var(1));
        assert_eq!(
            s.compose(&Substitution::new()).apply_to_type(&t),
            s.apply_to_type(&t)
        );
        assert_eq!(
            Substitution::new().compose(&s).apply_to_type(&t),
            s.apply_to_type(&t)
        );
    }

    #[test]
    fn compose_chain_in_hm_order() {
        // Riproduce `self.subst = self.subst.compose(&s)` come in hm.rs:
        // old subst {0 -> 1}, new s {1 -> Int}. Have to resolve 0 to Int.
        let acc = sub(&[(0, var(1))]);
        let s = sub(&[(1, Type::Int)]);
        assert_eq!(acc.compose(&s).apply_to_type(&var(0)), Type::Int);
    }

    #[test]
    fn compose_standard_order() {
        // s.compose(&acc)
        let acc = sub(&[(0, var(1))]);
        let s = sub(&[(1, Type::Int)]);
        assert_eq!(s.compose(&acc).apply_to_type(&var(0)), Type::Int);
    }
}