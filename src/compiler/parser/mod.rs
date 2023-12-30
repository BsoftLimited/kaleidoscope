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
    pub fn new(data:&str)->Self{
        let mut parser = Parser{ lexer: Box::new(Lexer::new(data)), errors: Vec::new() , current: Token::none() };
        parser.next_token();

        return  parser;
    }

    pub fn has_next(&mut self)->bool{ self.lexer.has_next() }
    
    fn next_token(&mut self)->bool{
        while self.lexer.has_next(){
            match self.lexer.get_next_token(){
                Err(error) =>{ self.errors.push(String::from(error)); }
                Ok(token) =>{
                    self.current = token;
                    println!("{:?}", self.current);
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
                    "let" => {
                        let init = self.initilaization();
                        if matches!(self.current.tokentype(), TokenType::SemiColon){
                            self.pop_token();
                        }else if init.is_some(){
                            self.errors.push(format!("Semicolon(;) expected after variable initialization"));   
                        }
                        return init;
                    },
                    //"fun" => return self.function_declaraton(),
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
            let exp = self.make_conditional();
            if exp.is_some(){
                output = Some(ExprAST::Assignment{ variable: Parser::unwrap(&init), exp: Box::new(exp.unwrap()) });
            }
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
                if step == 0{
                    if is_keyword(value.as_str()){
                        self.errors.push(format!("the word: {} is a reserve word expecting a {}", value, if step == 0 { "name" } else { "Data type" }));
                    }else if name.is_none(){
                        name = Some(value.clone());
                        step = 1;
                    }else{
                        self.errors.push(format!("the word: {} is not a Data type", value));
                    }
                }else if step == 2{
                    if data_type.is_none() && is_keyword(value.as_str()){
                        data_type = Option::from(value.clone());
                        step = 3;
                    }else if data_type.is_some() && is_keyword(value.as_str()){
                        self.errors.push(format!("unexpected Datatype({}) encountered", value));
                    }else if !is_keyword(&value.as_str()){
                        self.errors.push(format!("unexpected name({}) encountered", value));
                    }
                }
            }else if let TokenType::Colon = self.current.tokentype() {
                if step == 1{
                    step = 2;
                }else{
                    self.errors.push(format!("unexpected colon(:) encountered "));
                }
            }else if let TokenType::Equal = &self.current.tokentype(){
                if step == 3 || step == 1{
                    self.next_token();
                    let value = self.make_conditional();
                    if value.is_some(){
                        let dt = if step == 3 { data_type.unwrap() } else { "dynamic".to_owned() };
                        return Some(ExprAST::Initialization{ name: name.unwrap(), dt, value: Some(Box::new(value.unwrap())) });
                    }else{
                        self.errors.push(format!("value or expression expected after Equals(=)"));   
                    }
                }else{
                    self.errors.push(format!("unexpected Equals(=) encountered "));
                }
            }else if matches!(self.current.tokentype(), TokenType::Colon) || matches!(self.current.tokentype(), TokenType::SemiColon){
                if step == 3{
                    return Some(ExprAST::Initialization{ name: name.unwrap(), dt: data_type.unwrap(), value: None });
                }else if step == 1 {
                    return Some(ExprAST::Initialization{ name: name.unwrap(), dt: "dynamic".to_owned(), value: None });
                }else{
                    self.errors.push(format!("expected Variable Datatype specification or initialization before {:?}", self.current));
                }
            }else{
                if step == 3{
                    return Some(ExprAST::Initialization{ name: name.unwrap(), dt: data_type.unwrap(), value: None });
                }else if step == 2{
                    self.errors.push(format!("expected Datastype but found {:?}", self.current));
                }else if step == 1{
                    self.errors.push(format!("expected Colon(:) or Equals(=) = but found {:?}", self.current));
                }else {
                    self.errors.push(format!("expected Variable name but found {:?}", self.current));
                }
            }
        }
        return None;
    }

    /*fn get_argument_definition(&mut self)->Option<ExprAST>{
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
    }*/

    fn get_argument_passing(&mut self)->Vec<ExprAST>{
        let mut args: Vec<ExprAST> = Vec::new();
        let mut name: Option<String> = None;
        let mut step = 0;
        loop{
            if matches!(self.current.tokentype(), TokenType::Name(_)) || matches!(self.current.tokentype(), TokenType::Colon) || matches!(self.current.tokentype(), TokenType::Coma){
                let token = self.pop_token();
                if let TokenType::Colon = token.tokentype(){
                    if name.is_some() && step == 1{
                        let exp = self.make_conditional();
                        if exp.is_some(){
                            args.push( ExprAST::ArgumentPassing{ name: name.as_ref().unwrap().clone(), value: Box::new(exp.unwrap())});
                            step = 2;
                        }else{
                            self.errors.push(format!("expecting an argument a value after colon(:)"));
                        }
                    }else{
                        self.errors.push(format!("Unexpected column expecting an argument"));
                    }
                }else if let TokenType::Coma = token.tokentype(){
                    if step == 1 {
                        args.push( ExprAST::ArgumentPassing{ name: name.as_ref().unwrap().clone(), value: Box::new(ExprAST::Variable(name.as_ref().unwrap().clone()))});    
                    }
                    name = None;
                    step = 0;
                }else if let TokenType::Name(value) = token.tokentype(){
                    if name.is_some() {
                        if step == 2 && is_datatype(&value){

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
                if name.is_some() && matches!(self.current.tokentype(), TokenType::ClosingBracket){
                    args.push( ExprAST::ArgumentPassing{ name: name.as_ref().unwrap().clone(), value: Box::new(ExprAST::Variable(name.as_ref().unwrap().clone()))});
                }
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
            if matches!(self.current.tokentype(), TokenType::Name(_)) || matches!(self.current.tokentype(), TokenType::String(_)) || matches!(self.current.tokentype(), TokenType::Boolean(_)) || matches!(self.current.tokentype(), TokenType::Number(_)){
                let init = self.pop_token();
                if let TokenType::String(value) = init.tokentype(){
                    return Some(ExprAST::String(value.clone()));
                }else if let TokenType::Number(value) = init.tokentype(){
                    return Some(ExprAST::Number(value.clone() as f32));
                }if let TokenType::Name(value) = init.tokentype(){
                    println!("{}", value);
                    return Some(ExprAST::Variable(value.clone()));
                }
            }else{
                self.errors.push(format!("Unexpected token: {:?} expecting an string literal or boolean value", &self.current));
                self.next_token();
            }
        }
        return None;
    }

    pub fn print_errors(&self){
        for error in &self.errors{
            println!("{}", error);
        }
    }
}