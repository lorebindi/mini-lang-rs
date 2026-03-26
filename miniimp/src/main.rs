mod parser;
pub mod ast;
mod test;

fn main() {
    let program = r#"
        def main with input x output a as
            a := x + 1 + 2
            "#;
    if let Some(prog) = parser::parse_program(program) {
        println!("{:#?}", prog);
    }
}
