use crate::compiler::lexer::Token;

#[derive(Debug, Clone)]
pub enum ExprAST {
    Number(f32),
    Variable(String),
    Binary{ lhs: Box<ExprAST>, op:Token, rhs: Box<ExprAST> },
    Call{ callee: String, args: Vec<ExprAST> },
    Fucntion{ name: String, args: Vec<String>, body: Box<ExprAST> }
}