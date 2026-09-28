use std::collections::HashMap;
use std::rc::Rc;

/// A 'Scope<T>' forms an immutable/persistent chain of nested environments
/// linked to their enclosing parent scopes via reference counting ('Rc').
/// It serves two roles in the language pipeline:
/// - 'Context': mapping variable names to their static 'Type' during typechecking.
/// - 'Environment': mapping variable names to runtime 'Value' during evaluation.
#[derive(Clone)]
pub struct Scope<T> {
    pub(crate) bindings: HashMap<String, T>, // Bindings defined in the current scope.
    pub(crate) parent: Option<Rc<Scope<T>>>, // Reference to the enclosing scope,
                                     // None means that this is the outermost scope.
}

impl<T: Clone> Scope<T> {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    // Looks up a variable in the current scope and, if not found,
    // recursively searches the enclosing environments.
    pub fn lookup(&self, var: &str) -> Option<&T> {
        match self.bindings.get(var) {
            Some(value) => Some(value),
            None => match &self.parent {
                Some(parent) => parent.lookup(var),
                None => None,
            },
        }
    }

    // Creates a new scope extending the current one with
    // a new variable binding.
    pub fn extend(&self, var: String, value: T) -> Scope<T> {
        let mut bindings = HashMap::new();
        bindings.insert(var, value);

        Scope {
            bindings,
            parent: Some(Rc::new(self.clone())),
        }
    }
}

impl<T: Clone> Default for Scope<T> {
    fn default() -> Self {
        Self::new()
    }
}