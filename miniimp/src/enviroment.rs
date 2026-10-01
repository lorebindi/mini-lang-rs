//! Execution state and variable store for the 'miniimp' runtime environment.
//!
//! Provides the 'Environment' mapping identifier names ('String') to integer
//! values ('i64'), supporting operations to look up current variable bindings
//! and update memory during command execution.

use std::collections::HashMap;

pub struct Environment {
    bindings: HashMap<String, i64>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn lookup(&self, var: &str) -> i64 {
        *self.bindings
            .get(var)
            .unwrap_or_else(|| panic!("Undefined variable: {}", var))
    }

    pub fn update(&mut self, var: String, val: i64) {
        self.bindings.insert(var, val);
    }
}