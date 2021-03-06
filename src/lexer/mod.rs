pub mod tokens;

use crate::lexer::tokens::{Token, TokenType};
use crate::Result;

use std::iter::Peekable;
use std::vec::IntoIter;
use std::{fs, io};

/// A lexical analyzer that splits the program into [`Token`]s.
///
/// [`Token`]: tokens/enum.Token.html
pub struct Lexer {
    /// The raw program characters.
    raw_data: Peekable<IntoIter<char>>,
    pos: i32,
    line_no: i32,
}

impl Lexer {
    /// Create a lexer from a program file given the path to the file.
    ///
    /// # Arguments
    /// * `file_path` - The path to the program file.
    pub fn from_file(file_path: &str) -> io::Result<Self> {
        Ok(Self::from_text(&fs::read_to_string(file_path)?))
    }

    /// Create a lexer with the program data in plain text.
    ///
    /// # Arguments
    /// * `text` - The raw program.
    pub fn from_text(text: &str) -> Self {
        Lexer {
            raw_data: text.chars().collect::<Vec<_>>().into_iter().peekable(),
            pos: -1,
            line_no: 1,
        }
    }

    /// Create a token by eating characters while a condition is met.
    ///
    /// # Arguments
    /// * `raw_token` - The raw string token to append characters to.
    /// * `cond` - The condition that must be met.
    fn get_next_char_while(&mut self, raw_token: &mut String, cond: fn(char) -> bool) {
        loop {
            match self.raw_data.peek() {
                Some(c) if cond(*c) => {
                    if *c != '\n' {
                        self.pos += 1;
                    } else {
                        self.line_no += 1;
                        self.pos = 0;
                    };
                    raw_token.push(*c);
                    self.raw_data.next();
                }
                _ => {
                    break;
                }
            }
        }
    }

    /// Check if a character is a part of an identifier.
    ///
    /// Identifiers must start with an alphabetic character or underscore, and then can have
    /// alphanumeric characters and underscores.
    ///

    fn is_in_identifier(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
}

impl Iterator for Lexer {
    type Item = Result<Token>;

    /// Identifies the next token
    fn next(&mut self) -> Option<Self::Item> {
        let token: Result<TokenType>;
        let current_char: char;
        // Find first non-whitespace character
        loop {
            match self.raw_data.next() {
                Some(c) if (c == ' ' || c == '\t') => {
                    self.pos += 1;
                    continue;
                }
                Some(c) if c == '\n' => {
                    self.line_no += 1;
                    self.pos = 0;
                    continue;
                }
                // Comment
                Some(c) if c == '#' => {
                    let mut dump = String::new();
                    self.get_next_char_while(&mut dump, |c| c != '\n');
                    // println!("Lexing comment");
                    continue;
                }
                Some(c) => {
                    current_char = c;
                    self.pos += 1;
                    break;
                }
                None => return None,
            }
        }

        // println!("First char: {}", current_char);

        // Identifier
        if Self::is_in_identifier(current_char) && !current_char.is_numeric() {
            let mut name = current_char.to_string();
            self.get_next_char_while(&mut name, Self::is_in_identifier);
            match name {
                s if String::from("if") == s => token = Ok(TokenType::If),
                s if String::from("else") == s => token = Ok(TokenType::Else),
                s if String::from("let") == s => token = Ok(TokenType::Let),
                s if String::from("def") == s => token = Ok(TokenType::Def),
                s if String::from("extern") == s => token = Ok(TokenType::Extern),
                s if String::from("return") == s => token = Ok(TokenType::Return),
                s if String::from("true") == s => token = Ok(TokenType::True),
                s if String::from("false") == s => token = Ok(TokenType::False),
                s => {token = Ok(TokenType::Identifier(s))},
            };
        }
        // Integer Literal
        else if current_char.is_numeric() {
            let mut value = current_char.to_string();
            self.get_next_char_while(&mut value, |c| c.is_numeric());

            token = match value.parse() {
                Ok(i) => Ok(TokenType::Integer(i)),
                Err(_) => Err(format!("Integer literal {} is invalid", value)),
            }
        }
        // String Literal
        else if current_char == '"' {
            let mut value = String::new();

            self.get_next_char_while(&mut value, |c| c != '"');
            self.raw_data.next(); // Eat trailing "

            token = Ok(TokenType::Str(value));
        }
        // Semicolon
        else if current_char == ';' {
            token = Ok(TokenType::Semicolon);
        }
        // Colon
        else if current_char == ':' {
            token = Ok(TokenType::Colon);
        }
        // Comma
        else if current_char == ',' {
            token = Ok(TokenType::Comma);
        }
        // LParen
        else if current_char == '(' {
            token = Ok(TokenType::LParen);
        }
        // RParen
        else if current_char == ')' {
            token = Ok(TokenType::RParen);
        }
        // LBrack
        else if current_char == '[' {
            token = Ok(TokenType::LBrack);
        }
        // RBrack
        else if current_char == ']' {
            token = Ok(TokenType::RBrack);
        }
        // LBrace
        else if current_char == '{' {
            token = Ok(TokenType::LBrace);
        }
        // RBrace
        else if current_char == '}' {
            token = Ok(TokenType::RBrace);
        }
        // Plus and PlusEq
        else if current_char == '+' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::PlusEq);
            } else {
                token = Ok(TokenType::Plus);
            }
        }
        // Minus, Arrow and MinusEq
        else if current_char == '-' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::MinusEq);
            } else if self.raw_data.peek() == Some(&'>') {
                self.raw_data.next();
                token = Ok(TokenType::Arrow);
            } else {
                token = Ok(TokenType::Minus);
            }
        }
        // Mul and MulEq
        else if current_char == '*' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::MulEq);
            } else {
                token = Ok(TokenType::Mul);
            }
        }
        // Div and DivEq
        else if current_char == '/' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::DivEq);
            } else {
                token = Ok(TokenType::Div);
            }
        }
        // Less and LessEq
        else if current_char == '<' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::LessEq);
            } else {
                token = Ok(TokenType::Less);
            }
        }
        // Greater and GreaterEq
        else if current_char == '>' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::GreaterEq);
            } else {
                token = Ok(TokenType::GreaterEq);
            }
        }
        // Assign and Equal
        else if current_char == '=' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::Equal);
            } else {
                token = Ok(TokenType::Assign);
            }
        }
        // Not and NotEq
        else if current_char == '!' {
            if self.raw_data.peek() == Some(&'=') {
                self.raw_data.next(); // Eat =
                token = Ok(TokenType::NotEq);
            } else {
                token = Ok(TokenType::Not);
            }
        }
        
        else {
            token = Ok(TokenType::Unknown)
        }

        return Some(Ok(Token {
            type_: token.unwrap(),
            pos: self.pos,
            line_no: self.line_no,
        }));
    }
}

#[cfg(test)]
mod tests {

    use super::Lexer;

    #[test]
    fn is_in_identifier() {
        for &i in &['a', 'z', '_', '0', '9'] {
            assert!(Lexer::is_in_identifier(i));
        }

        for &s in &['+', '*', '@', ';'] {
            assert!(!Lexer::is_in_identifier(s));
        }
    }
}
