#[derive(Debug, Clone)]
pub struct Prototype{ pub name: String, pub args: Vec<ExprAST> }

#[derive(Debug, Clone)]
pub enum ExprAST {
    Number(f32),
    Variable(String),
    Binary{ lhs: Box<ExprAST>, op:String, rhs: Box<ExprAST> },
    Call{ callee: String, args: Vec<ExprAST> },
    Fucntion{ prototype: Box<Prototype>, body: Box<ExprAST> },
    None
}