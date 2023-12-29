use crate::compiler::lexer::Token;

#[derive(Debug, Clone)]
pub struct Prototype{ pub name: String, pub args: Vec<String> }

#[derive(Debug, Clone)]
pub enum ExprAST {
    Number(f32),
    Variable(String),
    Binary{ lhs: Box<ExprAST>, op:Token, rhs: Box<ExprAST> },
    Call{ callee: String, args: Vec<ExprAST> },
    Fucntion{ prototype: Box<Prototype>, body: Box<ExprAST> },
}