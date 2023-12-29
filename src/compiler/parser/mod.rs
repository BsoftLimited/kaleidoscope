mod ast;

use std::collections::HashMap;
use once_cell::sync::Lazy;

use self::ast::{ExprAST, Prototype};
use super::lexer::{Lexer, Token};

// BinopPrecedence - This holds the precedence for each binary operator that is defined.
static BINOP_PRECEDENCE: Lazy<HashMap<char, usize>> = Lazy::new(|| [ ('<', 10), ('+', 20), ('-', 30), ('*', 40) ].into());

/// Gettok_precedence - Get the precedence of the pending binary operator token.
fn get_tok_precedence(operator: &Option<Token>) -> isize{
    // Make sure it's a declared binop.
    if operator.is_some(){
        if let Token::Operator(op) = operator.as_ref().unwrap() {
            return BINOP_PRECEDENCE[op] as isize;   
        }
    }
    return -1;
}

pub struct Parser{
    lexer: Box<Lexer>,
    current: Option<Token>
}

impl Parser {
    pub fn new(data: &str)->Self{
        return Parser{
            lexer: Box::new(Lexer::new(data)),
            current: None
        };
    }

    fn get_next_token(&mut self){
        self.current = Some(self.lexer.gettok());
        println!("current  token: {:?}", self.current);
    }

    fn log_error<T>(&self, message: &str)->Option<T>{
        println!("{}", message);
        return None;
    }

    fn parse_expression(&mut self)->Option<ExprAST>{
        let lhs = self.parse_primary();
        if lhs.is_none(){
            return  lhs;
        }
        
        return self.parse_bin_op_rhs(0, lhs.unwrap());
    }

    pub fn parse_bin_op_rhs(&mut self, precedence: isize, in_lhs: ExprAST)-> Option<ExprAST>{
        let mut lhs = in_lhs;
        // If this is a binop, find its precedence.
        loop {
            let tok_prec = get_tok_precedence(&self.current);

            // If this is a binop that binds at least as tightly as the current binop,
            // consume it, otherwise we are done.
            if tok_prec < precedence{
                return Some(lhs);
            }

            // Okay, we know this is a binop.
            let bin_op = self.current.clone();
            self.get_next_token(); // eat binop

            // Parse the primary expression after the binary operator.
            let mut rhs = self.parse_primary();
            if rhs.is_none(){
                return None;
            }

            // If BinOp binds less tightly with RHS than the operator after RHS, let
            // the pending operator take RHS as its LHS.
            let next_prec = get_tok_precedence(&self.current);
            if tok_prec < next_prec {
                rhs = self.parse_bin_op_rhs(tok_prec + 1, rhs.unwrap());
                if rhs.is_none(){
                    return None;
                }
            }

            // Merge LHS/RHS.
            lhs = ExprAST::Binary{ lhs:Box::new(lhs), op: bin_op.unwrap().clone(), rhs: Box::new(rhs.unwrap())} ;
        }
    }

    fn parse_number_expr(&mut self, value: f32) -> ExprAST {
        let result =  ExprAST::Number(value);
        self.get_next_token(); // consume the number
        return result;
    }

    fn parse_paren_expr(&mut self) -> Option<ExprAST>{
        self.get_next_token(); // eat (.
        let v = self.parse_expression();
        if v.is_none(){
            return None;
        }

        if self.current.is_some() && matches!(self.current.as_ref().unwrap(), Token::CloseParenthesis){
            return self.log_error("expected ')'");
        }
          
        self.get_next_token(); // eat ).
        return v;
    }

    fn parse_identifier_expr(&mut self, name: String) -> Option<ExprAST>{
        self.get_next_token(); // eat identifier.

        if self.current.is_none() || !matches!(self.current.as_ref().unwrap(), Token::OpenParenthesis){ // Simple variable ref.
            return Some(ExprAST::Variable(name)); 
        }
      
        // Call.
        self.get_next_token();  // eat (

        let mut args: Vec<ExprAST> = Vec::new();
        if self.current.is_some() && matches!(self.current.as_ref().unwrap(), Token::CloseParenthesis){
            loop{
                if let Some(arg) = self.parse_expression(){
                    args.push(arg);
                }else{
                    return None;
                }

                if self.current.is_some() && matches!(self.current.as_ref().unwrap(), Token::CloseParenthesis){
                    break;
                }else if self.current.is_some() && matches!(self.current.as_ref().unwrap(), Token::Comma){
                    self.get_next_token();
                }else{
                    return self.log_error("Expected ')' or ',' in argument list");
                }
            }
        }
      
        // Eat the ')'.
        self.get_next_token();
      
        return Some(ExprAST::Call { callee: name, args });
    }

    /// prototype
    ///   ::= id '(' id* ')'
    fn parse_prototype(&mut self) -> Option<Prototype> {
        if let Token::Identifier(fn_name) =  self.current.clone().unwrap(){
            self.get_next_token();

            if self.current.is_none() || !matches!(self.current.as_ref().unwrap(), Token::OpenParenthesis){
                return self.log_error("Expected '(' in prototype");
            }
        
            // Read the list of argument names.
            let mut arg_names: Vec<String> = Vec::new();
            loop {
                self.get_next_token();
                if self.current.is_some(){
                    if let Token::Identifier(name) = self.current.as_ref().unwrap(){
                        arg_names.push(name.clone());
                    }else{
                        break;
                    }
                }else {
                    break;
                }
            }

            if self.current.is_none() || !matches!(self.current.as_ref().unwrap(), Token::CloseParenthesis){
                return self.log_error("Expected ')' in prototype");
            }
        
            // success.
            self.get_next_token();  // eat ')'.
        
            return Some(Prototype{ name: fn_name.to_owned(), args: arg_names });
        }else{
            return self.log_error("Expected function name in prototype");
        }
    }


    /// definition ::= 'def' prototype expression
    fn parse_definition(&mut self) ->Option<ExprAST> {
        self.get_next_token(); // eat def.

        let proto = self.parse_prototype();
        if proto.is_none(){
            return None;
        }

        let exp = self.parse_expression();
        if exp.is_none(){
            return None;
        }
    
        return Some(ExprAST::Fucntion{ prototype: Box::new(proto.unwrap()), body: Box::new(exp.unwrap()) });
    }
  

    /// external ::= 'extern' prototype
    fn parse_extern(&mut self) ->Option<Prototype> {
        self.get_next_token();  // eat extern.

        return self.parse_prototype();
    }

    fn parse_primary(&mut self) ->Option<ExprAST> {
        if self.current.is_some(){
            return match self.current.as_ref().unwrap() {
                Token::Identifier(name) => self.parse_identifier_expr(name.clone()),
                Token::Number(value) => Some(self.parse_number_expr(value.clone())),
                Token::OpenParenthesis => self.parse_paren_expr(),
                __ => self.log_error("unknown token when expecting an expression")
            }
        }
        self.log_error("unknown token when expecting an expression")
    }

    /// toplevelexpr ::= expression
    fn parse_top_level_expr(&mut self) ->Option<ExprAST> {
        let exp = self.parse_expression();
        if exp.is_some(){
            let proto = Prototype{ name: "".to_owned(), args: Vec::new() };

            return Some(ExprAST::Fucntion { prototype: Box::new(proto), body: Box::new(exp.unwrap()) });
        }
        return None;
    }

    //===----------------------------------------------------------------------===//
    // Top-Level parsing
    //===----------------------------------------------------------------------===//

    fn handle_definition(&mut self) {
        if self.parse_definition().is_some(){
            println!("Parsed a function definition.");
        }else{
            // Skip token for error recovery.
            self.get_next_token();
        }
    }
    
    fn handle_extern(&mut self) {
        if self.parse_extern().is_some(){
            println!("Parsed an extern.");
        }else{
            // Skip token for error recovery.
            self.get_next_token();
        }
    }
    
    fn handle_top_level_expression(&mut self) {
        if self.parse_top_level_expr().is_some(){
            println!("Parsed a top-level expr.");
        }else{
            // Skip token for error recovery.
            self.get_next_token();
        }
    }

    // top ::= definition | external | expression | ';'
    pub fn main_loop(&mut self) {
        self.get_next_token();
        loop {
            if self.current.is_some(){
                match self.current.as_ref().unwrap() {
                    Token::EndOfFile => break,
                    Token::SemiColons => self.get_next_token(),
                    Token::Def => self.handle_definition(),
                    Token::Extern => self.handle_extern(),
                    __ => self.handle_top_level_expression()
                }
            }else{
                break;
            }
        }
    }
}