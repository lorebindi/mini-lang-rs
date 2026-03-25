mod parser;
pub mod ast;

fn main() {
    let program = r#"
        def main with input x output a as
            a := x + (-1) + (1 + 1)
            "#;

    parser::parse_program(program);
}
