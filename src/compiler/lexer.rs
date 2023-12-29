use crate::utils::Character;

#[derive(Debug, Clone)]
pub enum Token{
    Number(f32),
    Identifier(String),
    Extern,
    Def,
    OpenParenthesis,
    CloseParenthesis,
    Comma,
    SemiColons,
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
                            '(' => {
                                self.index += 1;
                                return Token::OpenParenthesis;
                            },
                            ')' => {
                                self.index += 1;
                                return Token::CloseParenthesis;
                            },
                            ',' => {
                                self.index += 1;
                                return Token::Comma;
                            },
                            ';' => {
                                self.index += 1;
                                return Token::SemiColons;
                            },
                            '*' => {
                                self.index += 1;
                                return Token::Operator('*');
                            },
                            '-' => {
                                self.index += 1;
                                return Token::Operator('-');
                            },
                            '+' => {
                                self.index += 1;
                                return Token::Operator('+');
                            },
                            '<' => {
                                self.index += 1;
                                return Token::Operator('<');
                            },
                            _ => return self.init_token()
                        }
                    }
                    self.index += 1;
                },
                None=>{
                    return Token::EndOfFile;
                }
            }  
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

        if init == "def"{
            return Token::Def;
        }

        if init == "extern"{
            return  Token::Extern;
        }
        return Token::Identifier(init);
    }
}