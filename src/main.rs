mod compiler;
use compiler::Parser;
mod utils;

fn main() {
    println!("Hello, world!");

    let mut parser = Parser::new("def foo(x y) x+foo(y 4.0);");
    parser.main_loop();
}
