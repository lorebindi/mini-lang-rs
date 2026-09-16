use std::collections::HashMap;
use std::rc::Rc;

use crate::eval::Value;

#[derive(Clone)]
pub struct Environment {
    bindings: HashMap<String, Value>, // Bindings defined in the current scope.
    parent: Option<Rc<Environment>>, // Reference to the enclosing environment,
                                     // None means that this is the outermost environment.
}

impl Environment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    // Looks up a variable in the current scope and, if not found,
    // recursively searches the enclosing environments.
    pub fn lookup(&self, var: &str) -> Option<&Value> {
        match self.bindings.get(var) {
            Some(value) => Some(value),
            None => match &self.parent {
                Some(parent) => parent.lookup(var),
                None => None,
            },
        }
    }

    // Creates a new environment extending the current one with
    // a new variable binding.
    pub fn extend(&self, var: String, value: Value) -> Environment {
        let mut bindings = HashMap::new();
        bindings.insert(var, value);

        Environment {
            bindings,
            parent: Some(Rc::new(self.clone())),
        }
    }
}