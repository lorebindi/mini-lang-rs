/// This files defines the types used by MiniFun, distinguishing between monotypes and polytypes.
///
/// - 'Type' (Monotype): Quantifier-free types, including primitives ('int', 'bool'),
///   type variables, and function arrows ('t1 -> t2').
/// - 'TypeSchema' (Polytype)**: Rank-1 polymorphic types of the form '\forall α_1... \forall α_n. \tau',
///   with universal quantifiers restricted to the top level.
///
/// Provides fundamental operations: generalization ('generalize'),
/// instantiation ('inst'), and free variable extraction ('free_vars').

use std::collections::{HashMap, HashSet};
use std::fmt;
use super::substitution::{Substitution, FreshVarGen};
use super::errors::UnifyError;

pub type TypeVar = u32;

/// Monotypes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type{
    Int,
    Bool,
    Arrow(Box<Type>, Box<Type>), // t -> t'
    Var(TypeVar), // type variables identified by numbers
}

impl Type{
    
    /// Returns the set of free type variables occurring in this type.
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        match self {
            Type::Int | Type::Bool => HashSet::new(),
            Type::Arrow(t1, t2) => {
                let mut vars = t1.free_vars();
                vars.extend(t2.free_vars());
                vars
            }
            Type::Var(v) => {
                let mut vars = HashSet::new();
                vars.insert(*v);
                vars
            }
        }
    }

    /// Generalizes this type (monotype) into a TypeSchema (polytype) with respect to the given context,
    /// universally quantifying all free type variables not bound by the environment.
    pub fn generalize(&self, context_free_vars: &HashSet<TypeVar>) -> TypeSchema {
        let mut vars: Vec<TypeVar> = self.free_vars()
            .difference(&context_free_vars)
            .copied()
            .collect();
        vars.sort_unstable(); // deterministic order — matters for printing/tests, as discussed earlier

        TypeSchema { quantified_vars: vars, ty: self.clone() }
    }

    // helper function that bind 'var' to 'ty' producing a singleton substitution
    fn bind_var(var: TypeVar, ty: &Type) -> Result<Substitution, UnifyError> {
        if let Type::Var(v2) = ty {
            if *v2 == var {
                // If var and v2 are the same type variable then empty Substitution
                return Ok(Substitution::new());
            }
        }
        if ty.free_vars().contains(&var) {
            // if 'var' appears in the set of free variables of 'ty' would create cyclic types.
            return Err(UnifyError::OccursCheck { var, ty: ty.clone() });
        }
        let mut s = Substitution::new();
        s.insert(var, ty.clone());
        Ok(s)
    }

    /// Computes the most general substitution 'S' such that 'S(self) == S(other)'.
    pub fn unify(&self, other: &Type) -> Result<Substitution, UnifyError> {
        match (self, other) {
            (Type::Int, Type::Int) => Ok(Substitution::new()),
            (Type::Bool, Type::Bool) => Ok(Substitution::new()),

            (Type::Var(v1), Type::Var(v2)) if v1 == v2 => Ok(Substitution::new()),

            (Type::Var(v), _) => Self::bind_var(*v, other),
            (_, Type::Var(v)) => Self::bind_var(*v, self),

            (Type::Arrow(in1, out1), Type::Arrow(in2, out2)) => {
                // unify the arguments
                let s1 = in1.unify(in2)?;
                // apply s1 to out1 and out2 then unify
                let s2 = s1.apply_to_type(out1).unify(&s1.apply_to_type(out2))?;
                // unification of the substitution
                Ok(s1.compose(&s2))
            }
            _ => Err(UnifyError::Mismatch { expected: self.clone(), actual: other.clone() }),
        }
    }

    /// Performs a single-step simultaneous substitution on this type.
    ///
    /// Replaces occurrences of any 'TypeVar' found in 'map' with its corresponding 'Type'.
    /// The replacement types are not re-examined, avoiding transitive variable resolution
    /// and eliminating any risk of infinite cycles.
    fn substitute_once(&self, map: &HashMap<TypeVar, Type>) -> Type {
        match self {
            Type::Int | Type::Bool => self.clone(),
            Type::Arrow(t1, t2) => Type::Arrow(
                Box::new(t1.substitute_once(map)),
                Box::new(t2.substitute_once(map)),
            ),
            Type::Var(v) => map.get(v).cloned().unwrap_or_else(|| self.clone()),
        }
    }

}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Bool => write!(f, "bool"),
            Type::Arrow(arg, ret) => {
                // If the argument is itself a function, parentheses are needed
                // because -> is right-associative: (a -> b) -> c
                match **arg {
                    Type::Arrow(_, _) => write!(f, "({arg}) -> {ret}"),
                    _ => write!(f, "{arg} -> {ret}"),
                }
            }
            Type::Var(id) => {
                // print the ID as 'a, 'b, ..., or 'a0, 'a1
                if *id < 26 {
                    let c = (b'a' + *id as u8) as char;
                    write!(f, "'{c}")
                } else {
                    write!(f, "'a{id}")
                }
            }
        }
    }
}

/// Polytypes: \forall var_1. \forall var_2. \forall ... \forall var_n . (a valid type expressed using all the previous quantified variables)
/// Note 1: Quantifiers can only appear at the top-level, e.g. \forall α. α -> \forall α. α is not allowed
/// Note 2: Two polytypes are equal up-to reordering of the quantifiers and α-renaming
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSchema {
    pub quantified_vars: Vec<TypeVar>, // quantified variables
    pub ty: Type,  // the body of the type
}

impl TypeSchema {

    pub fn new_monomorphic(ty: Type) -> Self{
        Self {
            quantified_vars: Vec::new(),
            ty,
        }
    }

    pub fn new_polymorphic(ty: Type, quantified_vars: Vec<TypeVar>) -> Self{
        Self {
            quantified_vars,
            ty,
        }
    }

    /// Instantiates this type scheme by replacing all universally quantified variables
    /// with fresh type variables, producing a monotype ready for unification.
    pub fn inst(&self, fresh: &mut FreshVarGen) -> Type {
        let renaming: HashMap<TypeVar, Type> = self
            .quantified_vars
            .iter()
            .map(|&v| (v, Type::Var(fresh.fresh())))
            .collect();
        self.ty.substitute_once(&renaming)
    }

    /// Free variables of the schema: free variables of the body, minus
    /// whatever this schema itself quantifies over.
    pub fn free_vars(&self) -> HashSet<TypeVar> {
        let mut vars = self.ty.free_vars();
        for v in &self.quantified_vars {
            vars.remove(v);
        }
        vars
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn var(n: TypeVar) -> Type { Type::Var(n) }
    fn arrow(a: Type, b: Type) -> Type { Type::Arrow(Box::new(a), Box::new(b)) }
    fn set(vs: &[TypeVar]) -> HashSet<TypeVar> { vs.iter().copied().collect() }

    /// Unifies e verifies the fundamental property S(a) == S(b).
    fn unify_ok(a: &Type, b: &Type) -> Substitution {
        let s = a.unify(b).err().map_or_else(
            || a.unify(b).ok().unwrap(),
            |e| panic!("unification failed: {e:?}"),
        );
        assert_eq!(s.apply_to_type(a), s.apply_to_type(b));
        s
    }

    /// A 'FreshVarGen' thus fresh variables doesn't collide with the tests ones
    fn advanced_gen() -> FreshVarGen {
        let mut g = FreshVarGen::new();
        for _ in 0..100 { g.fresh(); }
        g
    }

    // free_vars (Type)

    #[test]
    fn free_vars_of_ground_types_is_empty() {
        assert!(Type::Int.free_vars().is_empty());
        assert!(Type::Bool.free_vars().is_empty());
        assert!(arrow(Type::Int, Type::Bool).free_vars().is_empty());
    }

    #[test]
    fn free_vars_of_var() {
        assert_eq!(var(3).free_vars(), set(&[3]));
    }

    #[test]
    fn free_vars_collects_from_nested_arrows_without_duplicates() {
        let t = arrow(var(0), arrow(var(1), arrow(var(0), Type::Int)));
        assert_eq!(t.free_vars(), set(&[0, 1]));
    }

    // generalize

    #[test]
    fn generalize_quantifies_all_vars_with_empty_context() {
        let s = arrow(var(1), var(0)).generalize(&HashSet::new());
        assert_eq!(s.quantified_vars, vec![0, 1]); // ordinate
        assert_eq!(s.ty, arrow(var(1), var(0)));
    }

    #[test]
    fn generalize_skips_context_vars() {
        let s = arrow(var(0), var(1)).generalize(&set(&[0]));
        assert_eq!(s.quantified_vars, vec![1]);
    }

    #[test]
    fn generalize_all_vars_in_context_is_monomorphic() {
        let s = arrow(var(0), var(1)).generalize(&set(&[0, 1]));
        assert!(s.quantified_vars.is_empty());
    }

    #[test]
    fn generalize_ground_type_has_no_quantifiers() {
        let s = Type::Int.generalize(&HashSet::new());
        assert!(s.quantified_vars.is_empty());
        assert_eq!(s.ty, Type::Int);
    }

    #[test]
    fn generalize_ignores_context_vars_not_in_type() {
        let s = var(0).generalize(&set(&[7, 8]));
        assert_eq!(s.quantified_vars, vec![0]);
    }

    // TypeSchema: constructors, free_vars

    #[test]
    fn monomorphic_schema_has_no_quantifiers() {
        let s = TypeSchema::new_monomorphic(var(0));
        assert!(s.quantified_vars.is_empty());
        assert_eq!(s.free_vars(), set(&[0]));
    }

    #[test]
    fn schema_free_vars_exclude_quantified() {
        // forall 0. 0 -> 1
        let s = TypeSchema::new_polymorphic(arrow(var(0), var(1)), vec![0]);
        assert_eq!(s.free_vars(), set(&[1]));
    }

    #[test]
    fn fully_quantified_schema_has_no_free_vars() {
        let s = TypeSchema::new_polymorphic(arrow(var(0), var(0)), vec![0]);
        assert!(s.free_vars().is_empty());
    }

    // inst

    #[test]
    fn inst_monomorphic_returns_same_type() {
        let s = TypeSchema::new_monomorphic(arrow(var(0), Type::Int));
        assert_eq!(s.inst(&mut advanced_gen()), arrow(var(0), Type::Int));
    }

    #[test]
    fn inst_renames_quantified_var_consistently() {
        // \forall 0. 0 -> 0   ==>   v -> v with fresh v
        let s = TypeSchema::new_polymorphic(arrow(var(0), var(0)), vec![0]);
        match s.inst(&mut advanced_gen()) {
            Type::Arrow(a, b) => {
                assert_eq!(a, b);
                assert_ne!(*a, var(0));
            }
            other => panic!("attended Arrow, found {other:?}"),
        }
    }

    #[test]
    fn inst_uses_distinct_fresh_var_per_quantifier() {
        // \forall 0 1. 0 -> 1
        let s = TypeSchema::new_polymorphic(arrow(var(0), var(1)), vec![0, 1]);
        match s.inst(&mut advanced_gen()) {
            Type::Arrow(a, b) => assert_ne!(a, b),
            other => panic!("attended Arrow, found {other:?}"),
        }
    }

    #[test]
    fn inst_twice_gives_different_variables() {
        let s = TypeSchema::new_polymorphic(var(0), vec![0]);
        let mut g = advanced_gen();
        assert_ne!(s.inst(&mut g), s.inst(&mut g));
    }

    #[test]
    fn inst_leaves_free_vars_untouched() {
        // forall 0. 0 -> 5   (5 è libera)
        let s = TypeSchema::new_polymorphic(arrow(var(0), var(5)), vec![0]);
        match s.inst(&mut advanced_gen()) {
            Type::Arrow(_, ret) => assert_eq!(*ret, var(5)),
            other => panic!("attended Arrow, found {other:?}"),
        }
    }

    #[test]
    fn inst_is_correct_when_fresh_vars_collide_with_quantified() {
        // forall 0 1. 0 -> 1, generatore fermo a 1: atteso v1 -> v2
        let mut g = FreshVarGen::new();
        g.fresh(); // consuma 0, il prossimo e' 1
        let s = TypeSchema::new_polymorphic(arrow(var(0), var(1)), vec![0, 1]);
        assert_eq!(s.inst(&mut g), arrow(var(1), var(2)));
    }

    // unify: base case

    #[test]
    fn unify_same_primitives() {
        unify_ok(&Type::Int, &Type::Int);
        unify_ok(&Type::Bool, &Type::Bool);
    }

    #[test]
    fn unify_int_bool_mismatch() {
        let r = Type::Int.unify(&Type::Bool);
        assert!(matches!(
            r,
            Err(UnifyError::Mismatch { expected, actual })
                if expected == Type::Int && actual == Type::Bool
        ));
    }

    #[test]
    fn unify_primitive_with_arrow_mismatch() {
        assert!(matches!(
            Type::Int.unify(&arrow(Type::Int, Type::Int)),
            Err(UnifyError::Mismatch { .. })
        ));
        assert!(matches!(
            arrow(Type::Int, Type::Int).unify(&Type::Bool),
            Err(UnifyError::Mismatch { .. })
        ));
    }

    // unify: variables

    #[test]
    fn unify_var_with_itself_is_empty() {
        let s = unify_ok(&var(0), &var(0));
        assert_eq!(s.apply_to_type(&var(0)), var(0));
    }

    #[test]
    fn unify_var_with_type_binds_it() {
        let s = unify_ok(&var(0), &Type::Int);
        assert_eq!(s.apply_to_type(&var(0)), Type::Int);
    }

    #[test]
    fn unify_type_with_var_binds_it() {
        let s = unify_ok(&Type::Bool, &var(0));
        assert_eq!(s.apply_to_type(&var(0)), Type::Bool);
    }

    #[test]
    fn unify_two_distinct_vars() {
        let s = unify_ok(&var(0), &var(1));
        assert_eq!(s.apply_to_type(&var(0)), s.apply_to_type(&var(1)));
    }

    #[test]
    fn unify_var_with_arrow_binds_whole_arrow() {
        let t = arrow(Type::Int, var(1));
        let s = unify_ok(&var(0), &t);
        assert_eq!(s.apply_to_type(&var(0)), t);
    }

    // unify: occurs check

    #[test]
    fn occurs_check_var_inside_arrow() {
        let r = var(0).unify(&arrow(var(0), Type::Int));
        assert!(matches!(r, Err(UnifyError::OccursCheck { var: 0, .. })));
    }

    #[test]
    fn occurs_check_is_symmetric() {
        let r = arrow(Type::Int, var(0)).unify(&var(0));
        assert!(matches!(r, Err(UnifyError::OccursCheck { var: 0, .. })));
    }

    #[test]
    fn occurs_check_deeply_nested() {
        let t = arrow(Type::Int, arrow(Type::Bool, var(0)));
        assert!(matches!(
            var(0).unify(&t),
            Err(UnifyError::OccursCheck { .. })
        ));
    }

    // unify: arrow

    #[test]
    fn unify_identical_arrows() {
        let t = arrow(Type::Int, Type::Bool);
        unify_ok(&t, &t);
    }

    #[test]
    fn unify_arrow_binds_both_sides() {
        // (a -> b) ~ (Int -> Bool)
        let s = unify_ok(&arrow(var(0), var(1)), &arrow(Type::Int, Type::Bool));
        assert_eq!(s.apply_to_type(&var(0)), Type::Int);
        assert_eq!(s.apply_to_type(&var(1)), Type::Bool);
    }

    #[test]
    fn unify_arrow_propagates_first_binding_to_second() {
        // (a -> a) ~ (Int -> b)   =>   a = Int, b = Int
        let s = unify_ok(&arrow(var(0), var(0)), &arrow(Type::Int, var(1)));
        assert_eq!(s.apply_to_type(&var(0)), Type::Int);
        assert_eq!(s.apply_to_type(&var(1)), Type::Int);
    }

    #[test]
    fn unify_arrow_chained_bindings() {
        // (a -> b) ~ (b -> Int)   =>   a = b = Int
        // Stress the composition s1.compose(&s2) inside unify.
        let s = unify_ok(&arrow(var(0), var(1)), &arrow(var(1), Type::Int));
        assert_eq!(s.apply_to_type(&var(0)), Type::Int);
        assert_eq!(s.apply_to_type(&var(1)), Type::Int);
    }

    #[test]
    fn unify_arrow_argument_mismatch() {
        let r = arrow(Type::Int, Type::Int).unify(&arrow(Type::Bool, Type::Int));
        assert!(matches!(r, Err(UnifyError::Mismatch { .. })));
    }

    #[test]
    fn unify_arrow_return_mismatch_after_first_binding() {
        // (a -> a) ~ (Int -> Bool): first a = Int, then Int vs Bool
        let r = arrow(var(0), var(0)).unify(&arrow(Type::Int, Type::Bool));
        assert!(matches!(
            r,
            Err(UnifyError::Mismatch { expected, actual })
                if expected == Type::Int && actual == Type::Bool
        ));
    }

    #[test]
    fn unify_arrow_occurs_check_across_components() {
        // (a -> b) ~ (b -> (a -> Int)) : a = b, poi b ~ (b -> Int) => occurs check
        let r = arrow(var(0), var(1))
            .unify(&arrow(var(1), arrow(var(0), Type::Int)));
        assert!(matches!(r, Err(UnifyError::OccursCheck { .. })));
    }

    #[test]
    fn unify_higher_order_arrows() {
        // ((a -> b) -> a) ~ ((Int -> Bool) -> c)
        let s = unify_ok(
            &arrow(arrow(var(0), var(1)), var(0)),
            &arrow(arrow(Type::Int, Type::Bool), var(2)),
        );
        assert_eq!(s.apply_to_type(&var(2)), Type::Int);
    }

    // Display

    #[test]
    fn display_primitives() {
        assert_eq!(Type::Int.to_string(), "int");
        assert_eq!(Type::Bool.to_string(), "bool");
    }

    #[test]
    fn display_type_vars() {
        assert_eq!(var(0).to_string(), "'a");
        assert_eq!(var(25).to_string(), "'z");
        assert_eq!(var(26).to_string(), "'a26");
    }

    #[test]
    fn display_arrow_right_associative_without_parens() {
        let t = arrow(Type::Int, arrow(Type::Bool, Type::Int));
        assert_eq!(t.to_string(), "int -> bool -> int");
    }

    #[test]
    fn display_arrow_parenthesizes_function_argument() {
        let t = arrow(arrow(Type::Int, Type::Bool), Type::Int);
        assert_eq!(t.to_string(), "(int -> bool) -> int");
    }
}