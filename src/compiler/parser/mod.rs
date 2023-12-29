mod ast;
pub use ast::{ ExprAST, Prototype };

mod syntax;
use syntax::*;

use super::lexer::{Token, Lexer, TokenType};

fn variant_equal(a: &Token, b: &Token)->bool{
    return std::mem::discriminant(&a.tokentype()) == std::mem::discriminant(&b.tokentype());
}

pub struct Parser{ lexer:Box<Lexer>, errors: Vec<String> , current: Token }
impl Parser{
    pub fn new(data:&str)->Result<Self, String>{
        let mut parser = Parser{ lexer: Box::new(Lexer::new(data)), errors: Vec::new() , current: Token::none() };
        if parser.next_token(){
            return Ok(parser);
        }
        return Err("There no source code to compile".to_owned());
    }

    pub fn has_next(&mut self)->bool{ self.lexer.has_next() }
    
    fn next_token(&mut self)->bool{
        while self.lexer.has_next(){
            match self.lexer.get_next_token(){
                Err(error) =>{ self.errors.push(String::from(error)); }
                Ok(token) =>{
                    self.current = token;
                    return true;
                }
            }
        }
        self.current = Token::none();
        return false;
    }

    fn pop_token(&mut self)->Token{
        let init = self.current.clone();
        self.next_token();
        return init;
    }

    fn unwrap(token:&Token)->String{
        let mut init = String::new();
        if let TokenType::Name(value) = token.tokentype(){
            init = value.clone();
        }else if let TokenType::String(value) = token.tokentype(){
            init = value.clone();
        } 
        init
    }

    pub fn get_next(&mut self)->Option<ExprAST>{
        while !matches!(self.current.tokentype(), TokenType::None){
            if let TokenType::Name(name) = self.current.tokentype(){
                match name.as_str(){
                    "let" => return self.initilaization(),
                    "fun" => return self.function_declaraton(),
                    _ => return self.get()
                }
            }
            let token = self.pop_token();
            self.errors.push(format!("Unexpected token: {:?}", token));
        }
        return None;
    }

    fn get(&mut self)-> Option<ExprAST>{
        let init = self.pop_token();
        let mut output: Option<ExprAST> = None;
        if let TokenType::OpenBracket = self.current.tokentype(){
            self.pop_token();
            output = Some(ExprAST::Call{ callee: Parser::unwrap(&init), args: self.get_argument_passing() });
        }else if let TokenType::Equal = self.current.tokentype(){
            self.pop_token();
            output = Some(ExprAST::Assignment{ variable: Parser::unwrap(&init), exp: Box::new(self.make_conditional()) });
        }

        if !matches!(self.current.tokentype(), TokenType::SemiColon){
            self.errors.push(format!("expected a semi colon ';'"));
        }
        return output;
    }

    fn initilaization(&mut self)->Option<ExprAST>{
        let mut name: Option<String> = None;
        let mut data_type: Option<String> = None;
        let mut step: u8 = 0;

        while self.next_token(){
            if let TokenType::Name(value) = self.current.tokentype() {
                if is_keyword(value.as_str()) && step == 0{
                    self.errors.push(format!("the word: {} is a reserve word expecting a {}", value, if step == 0 { "name" } else { "Data type" }));
                }else if  name.is_none() && step == 0 {
                    name = Option::from(value.clone());
                    step = 1;
                }else if data_type.is_none() && step == 2 && is_datatype(value.as_str()){
                    data_type = Option::from(value.clone());
                    step = 3;
                }
            }else if let TokenType::Colon = self.current.tokentype() {
                if step == 1{
                    step = 2;
                }
            }else if let TokenType::Equal = &self.current.tokentype(){
                if step == 3{
                    self.next_token();
                    let value = Some(Box::new(self.make_conditional()));
                    if matches!(self.current.tokentype(), TokenType::SemiColon){
                        self.next_token();
                        return Expression::Variable{ name: name.unwrap(), dt: data_type.unwrap(), value };
                    }
                }
            }else if let TokenType::SemiColon = self.current.tokentype() {
                if step == 3 {
                    self.next_token();
                    return Expression::Variable{ name: name.unwrap(), dt: data_type.unwrap(), value: None};
                }
            }
        }
        return Expression::None;
    }

    fn get_argument_definition(&mut self)->Option<ExprAST>{
        let name: String = Parser::unwrap(&self.current);
        let mut dt: Option<String> = None;
        let mut value: Option<Box<Expression>> = None;
        let mut step: u8 = 0;
        while self.next_token(){
            if let TokenType::Name(value) = self.current.tokentype(){
                if dt.is_none() && is_datatype(value.as_str()) && step == 1{
                    dt = Option::from(value.clone());
                    step = 2;
                }else if is_keyword(value.as_str()){
                    self.errors.push(format!("the word: {} is a reserve word expecting a {}", value, if step == 0 { "name" } else { "Data type" }));
                }
            }else if let TokenType::Colon = self.current.tokentype(){
                if step == 0{ step = 1; }
            }else if let TokenType::Equal = self.current.tokentype(){
                if step == 2{
                    value = Some(Box::new(self.make_conditional()));
                    step = 3;
                }
            }else if matches!(self.current.tokentype(), TokenType::Coma) || matches!(self.current.tokentype(), TokenType::ClosingBracket){
                if step == 2 || step == 3{
                    return Expression::ArgumentDefinition{ name, dt: dt.as_ref().unwrap().clone(), value };
                }
            }
        }
        return Expression::None;
    }

    fn function_declaraton(&mut self)->Option<ExprAST>{
        let mut args: Vec<Expression> = Vec::new();
        let mut body: Vec<Expression> = Vec::new();
        let mut name: Option<String> = None;
        let mut rtype: Option<String> = None;
        let mut step = 0;

        while self.next_token(){
            if let TokenType::Name(value) = self.current.tokentype() {
                if step == 0 && is_keyword(value.as_str()){
                    self.errors.push(format!("the word: {} is a reserve word expecting a {}", value, if step == 0 { "name" } else { "Data type" }));
                }else if  step == 0 && name.is_none() {
                    name = Option::from(value.clone());
                    step = 1;
                }else if rtype.is_none() && step == 4 && is_datatype(value.as_str()){
                    rtype = Option::from(value.clone());
                    step = 5;
                }else if step == 2{
                    args.push(self.get_argument_definition());
                }
            }else if let TokenType::OpenBracket = self.current.tokentype() {
                if step == 1{ step = 2; }
            }else if let TokenType::ClosingBracket = self.current.tokentype() {
                if step == 2{ step = 3; }
            }else if let TokenType::Colon = self.current.tokentype() {
                if step == 3{ step = 4; }
            }else if let TokenType::OpenCurlyBracket = self.current.tokentype() {
                if step == 3 || step == 5{
                    self.next_token();
                    while !(matches!(self.current.tokentype(), TokenType::ClosingCurlyBracket) || matches!(self.current.tokentype(), TokenType::None)){
                        body.push(self.get_next());
                    }
                    if let TokenType::None = self.current.tokentype() {
                        self.errors.push(format!("Unexpected end of tokens expecting a closing bracket ')'"));
                    }
                    return Expression::FunctionDefinition{ name:name.unwrap(), args, rtype, body };
                }
            }
        }
        return Expression::None;
    }

    fn get_argument_passing(&mut self)->Vec<ExprAST>{
        let mut args: Vec<ExprAST> = Vec::new();
        let mut name: Option<String> = None;
        let mut step = 0;
        loop{
            if matches!(self.current.tokentype(), TokenType::Name(_)) || matches!(self.current.tokentype(), TokenType::Colon) || matches!(self.current.tokentype(), TokenType::Coma){
                let token = self.pop_token();
                if let TokenType::Colon = token.tokentype(){
                    if name.is_some() && step == 1{
                        //args.push( ExprAST::ArgumentPassing{ name: name.as_ref().unwrap().clone(), value: Box::new(self.make_conditional())});
                        step = 2;
                    }else{
                        self.errors.push(format!("Unexpected column expecting an argument"));
                    }
                }else if let TokenType::Coma = token.tokentype(){
                    if step == 1 {
                        if name.is_some(){
                            self.errors.push(format!("Unexpected token: {:?} exprcting colon(:)", self.current));    
                        }
                    }
                    name = None;
                    step = 0;
                }else if let TokenType::Name(value) = token.tokentype(){
                    if name.is_some() {
                        if step == 2 && is_datatype(value){

                        }if step == 1{
                            self.errors.push(format!("Unexpected token: {:?} expecting a column(:)", &name));
                        }
                    }else if step == 0 {
                        if is_keyword(value.as_str()){
                            self.errors.push(format!("the word: {} is a reserve word", value));
                        }else{
                            name = Some(value.clone());
                            step = 1;
                        }
                    } 
                }
            } else if matches!(self.current.tokentype(), TokenType::ClosingBracket) || matches!(self.current.tokentype(), TokenType::None){
                self.next_token();
                break;
            } else{
                let token = self.pop_token();
                self.errors.push(format!("Unexpected token: {:?}", token));
            }
        }
        return args;
    }

    pub fn make_conditional(&mut self)->Option<ExprAST>{
        let left = self.make_term();
        if let TokenType::Conditional(op) = self.current.tokentype().clone(){
            self.next_token();
            let rhs = self.make_conditional();
            if rhs.is_some(){
                return Some(ExprAST::Binary{ lhs: Box::new(left.unwrap()), op, rhs: Box::new(rhs.unwrap()) });
            }
        }
        return left;
    }

    pub fn make_term(&mut self)->Option<ExprAST>{
        let left = self.make_factor();
        if let TokenType::Term(op) = self.current.tokentype().clone(){
            self.next_token();
            let rhs = self.make_conditional();
            if rhs.is_some(){
                return Some(ExprAST::Binary{ lhs: Box::new(left.unwrap()), op: String::from(op), rhs: Box::new(rhs.unwrap()) });
            }
        }
        return left;
    }

    pub fn make_factor(&mut self)->Option<ExprAST>{
        let left = self.make_value();
        if let TokenType::Factor(op) = self.current.tokentype().clone(){
            self.next_token();
            let rhs = self.make_conditional();
            if rhs.is_some(){
                return Some(ExprAST::Binary{ lhs: Box::new(left.unwrap()), op: String::from(op), rhs: Box::new(rhs.unwrap()) });
            }
        }
        return left;
    }

    fn make_value(&mut self)->Option<ExprAST>{
        while !matches!(self.current.tokentype(), TokenType::None){
            if matches!(self.current.tokentype(), TokenType::String(_)) || matches!(self.current.tokentype(), TokenType::Boolean(_)) || matches!(self.current.tokentype(), TokenType::Number(_)){
                let init = self.pop_token();
                if let TokenType::String(value) = init.tokentype(){
                    return Some(ExprAST::Variable(value.clone()));
                }else if let TokenType::Number(value) = init.tokentype(){
                    return Some(ExprAST::Number(value.clone() as f32));
                }
            }else{
                self.errors.push(format!("Unexpected token: {:?} expecting an string literal or boolean value", &self.current));
                self.next_token();
            }
        }
        return ExprAST::None;
    }

    pub fn print_errors(&self){
        for error in &self.errors{
            println!("{}", error);
        }
    }
}