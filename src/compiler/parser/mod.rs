mod ast;

use std::collections::HashMap;

use self::ast::ExprAST;

use super::lexer::{Lexer, Token};

struct Parser{
    lexer: Box<Lexer>,
    current: Option<Token>,
    binop_precedence: HashMap<char, usize> // BinopPrecedence - This holds the precedence for each binary operator that is defined.
}

impl Parser {
    pub fn new(data: &str)->Self{
        return Parser{
            lexer: Box::new(Lexer::new(data)),
            current: None,
            binop_precedence: [ ('<', 10), ('+', 20), ('-', 30), ('*', 40) ].into()
        };
    }

    pub fn get_next_token(&mut self){
        self.current = Some(self.lexer.gettok());
    }

    fn log_error(&self, message: &str)->Option<ExprAST>{
        println!("{}", message);
        return None;
    }

    /// GetTokPrecedence - Get the precedence of the pending binary operator token.
    fn get_tok_precedence(&self) -> isize{
        // Make sure it's a declared binop.
        if self.current.is_some(){
            if let Token::Operator(op) =  self.current.as_ref().unwrap(){
                return self.binop_precedence[op] as isize;
            }
        }
        return -1;
    }

    pub fn parse_expression(&mut self)->Option<ExprAST>{
        let lhs = self.parse_primary();
        if lhs.is_none(){
            return  lhs;
        }
        
        return self.parse_bin_op_rhs(0, lhs.unwrap());
    }

    pub fn parse_bin_op_rhs(&self, precedence: isize, lhs: ExprAST)-> Option<ExprAST>{

        return None;
    }

    pub fn parse_number_expr(&mut self, value: f32) -> ExprAST {
        let result =  ExprAST::Number(value);
        self.get_next_token(); // consume the number
        return result;
    }

    pub fn parse_paren_expr(&mut self) -> Option<ExprAST>{
        self.get_next_token(); // eat (.
        let v = self.parse_expression();
        if v.is_some(){
            return None;
        }

        if self.current.is_some() && matches!(self.current.as_ref().unwrap(), Token::CloseParenthesis){
            return self.log_error("expected ')'");
        }
          
        self.get_next_token(); // eat ).
        return v;
    }

    pub fn parse_identifier_expr(&mut self, name: String) -> Option<ExprAST>{
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
      
        return Some(ExprAST::Call { callee: name, args: args });
    }

    pub fn parse_primary(&mut self) ->Option<ExprAST> {
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
}