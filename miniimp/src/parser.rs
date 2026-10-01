//! Concrete syntax parsing and AST construction frontend for 'miniimp'.
//!
//! Exposes top-level parsing routines while encapsulating grammar rules and
//! internal CST walking passes.

pub mod parser;

pub use parser::parse_program;