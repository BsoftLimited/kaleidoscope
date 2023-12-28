mod ast;

use std::collections::HashMap;
use once_cell::sync::Lazy;

use self::ast::ExprAST;
use super::lexer::{Lexer, Token};

// BinopPrecedence - This holds the precedence for each binary operator that is defined.
static BINOP_PRECEDENCE: Lazy<HashMap<char, usize>> = Lazy::new(|| [ ('<', 10), ('+', 20), ('-', 30), ('*', 40) ].into());

/// Gettok_precedence - Get the precedence of the pending binary operator token.
fn get_tok_precedence(operator: &Option<Token>) -> isize{
    // Make sure it's a declared binop.
    if operator.is_some(){
        if let Token::Operator(op) =  operator.as_ref().unwrap(){
            return BINOP_PRECEDENCE[op] as isize;
        }
    }
    return -1;
}

struct Parser{
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

    pub fn get_next_token(&mut self){
        self.current = Some(self.lexer.gettok());
    }

    fn log_error(&self, message: &str)->Option<ExprAST>{
        println!("{}", message);
        return None;
    }

    pub fn parse_expression(&mut self)->Option<ExprAST>{
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
            let bin_op = self.current.as_ref().clone();
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
            lhs = ExprAST::Binary(Box::new(lhs), bin_op.as_deref().unwrap(), Box::new(rhs.unwrap()));
        }
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