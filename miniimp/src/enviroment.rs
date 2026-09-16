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