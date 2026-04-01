mod parser;
pub mod ast;

fn main() {
    let program = r#"
        def main with input x output a as
            a := ap
            "#;
    if let Some(prog) = parser::parse_program(program) {
        println!("{:#?}", prog);
    }
    else{
        println!("The program can't be parsed.");
    }
}
