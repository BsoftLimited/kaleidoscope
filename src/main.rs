mod compiler;
use compiler::Parser;

use crate::compiler::Lexer;
mod utils;

fn main() {
    println!("Hello, world!");

    let code = "bobby(x:3, y:4, z: 5, error )";

    /*let mut lexer = Lexer::new(code);
    while lexer.has_next(){
        println!("{:?}", lexer.get_next_token());
    }*/

    let mut parser = Parser::new(code);
    while parser.has_next(){
        let exp = parser.get_next();
        if exp.is_some(){
            println!("{:?}", exp.unwrap());
        }else {
            parser.print_errors();
            break;
        }
    }
}
