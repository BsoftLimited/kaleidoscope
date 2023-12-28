

use std::collections::HashMap;


use crate::utils::Character;

pub enum Token{
    Number(f32),
    Identifier(String),
    Extern,
    Def,
    OpenParenthesis,
    CloseParenthesis,
    Comma,
    Operator(char),
    EndOfFile,
}

pub struct Lexer{ index: usize, data: String }

impl Lexer {
    pub fn new(data: &str)->Self{
        return Lexer{ data: String::from(data), index: 0 }
    }

    pub fn gettok(&mut self)-> Token{
        loop {
            match self.data.chars().nth(self.index){
                Some(current_char) =>{
                    let current = Character::new(current_char);
                    if !current.is_whitespace(){
                        match current.unwrap(){
                            '(' => return Token::OpenParenthesis,
                            ')' => return Token::CloseParenthesis,
                            ',' => return Token::Comma,
                            _ => return self.init_token()
                        }
                    }
                },
                None=>{
                    return Token::EndOfFile;
                }
            }
            self.index += 1;   
        }
    }

    fn init_token(&mut self)->Token{
        let init = Character::new(self.data.chars().nth(self.index).unwrap());
        if init.is_alphabetic(){
            return self.init_indentifier_token();
        }else if init.is_numeric(){
            return self.init_numeric_token();
        }
        panic!("unknown token encountered: {}", init.unwrap());
    }

    fn init_numeric_token(&mut self)->Token{
        let mut init = String::new();
        loop {
            if let Some(current_char)  = self.data.chars().nth(self.index){
                let current = Character::new(current_char);
                if current.is_numeric(){
                    init.push(current_char);
                }else{
                    break;
                }
            }else{
                break;
            }
            self.index += 1;
        }
        return Token::Number(init.parse::<f32>().expect("cannot convert string into number"));
    }

    fn init_indentifier_token(&mut self)->Token{
        let mut init = String::new();
        loop {
            if let Some(current_char)  = self.data.chars().nth(self.index){
                let current = Character::new(current_char);
                if current.is_alphanumeric(){
                    init.push(current_char);
                }else{
                    break;
                }
            }else{
                break;
            }
            self.index += 1;
        }
        return Token::Identifier(init);
    }
}