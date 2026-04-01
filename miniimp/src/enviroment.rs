use std::collections::HashMap;

pub type Env = HashMap<String, i64>;

pub fn lookup(env: &Env, var: &str) -> i64 {
    *env.get(var).expect(&format!("Variabile non definita: {}", var))
}

pub fn update(env: &mut Env, var: String, val: i64) {
    env.insert(var, val);
}