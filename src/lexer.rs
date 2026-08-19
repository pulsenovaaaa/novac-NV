use std::{iter::Peekable, str::Chars};

use crate::Token;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    fn skip_whitespaces(&mut self) {
        while let Some(&ch) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        match ident.as_str() {
            "exit" =>       Token::KeywordExit,
            "set" =>        Token::KeywordSet,
            "if" =>         Token::KeywordIf,
            "else" =>       Token::KeywordElse,
            "putchar" =>    Token::KeywordPutChar,
            "rtm_print" =>  Token::CRuntimeKeywordPrint,
            _ =>            Token::Identifier(ident),
        }
    }

    fn read_number(&mut self) -> Token {
        let mut number_str = String::new();
        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                number_str.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        let val = number_str.parse::<i64>().unwrap_or(0);
        Token::IntLiteral(val)
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, String> {
        self.skip_whitespaces();

        let &ch = self.chars.peek().ok_or("Unexpected end of input")?;

        let token = match ch {
            '(' => {
                self.chars.next();
                Token::OpenParen
            }
            ')' => {
                self.chars.next();
                Token::CloseParen
            }
            ';' => {
                self.chars.next();
                Token::SemiColon
            }
            '=' => {
                self.chars.next();
                if self.chars.peek() == Some(&'=') {
                    self.chars.next();
                    Token::EqualsEquals
                } else {
                    Token::Equals
                }
            }
            ':' => {
                self.chars.next();
                Token::Colon
            }
            '+' => {
                self.chars.next();
                Token::BinPlus
            }
            '-' => {
                self.chars.next();
                Token::BinMinus
            }
            '*' => {
                self.chars.next();
                Token::BinMul
            }
            '/' => {
                self.chars.next();
                match self.chars.peek() {
                    Some(&'/') => {
                        while let Some(c) = self.chars.next() {
                            if c == '\n' || c == '\r' {
                                break;
                            }
                        }
                        return self.next_token();
                    }
                    _ => Token::BinDiv,
                }
            }
            '<' => {
                self.chars.next();
                if self.chars.peek() == Some(&'=') {
                    self.chars.next();
                    Token::LogicalLessEqual
                } else {
                    Token::LogicalLess
                }
            },
            '>' => {
                self.chars.next();
                if self.chars.peek() == Some(&'=') {
                    self.chars.next();
                    Token::LogicalGreaterEqual
                } else {
                    Token::LogicalGreater
                }
            },
            '{' => {
                self.chars.next();
                Token::OpenCurly
            },
            '}' => {
                self.chars.next();
                Token::CloseCurly
            },
            '!' => {
                self.chars.next();
                if self.chars.peek() == Some(&'=') {
                    self.chars.next();
                    Token::LogicalNotEqual
                } else {
                    Token::Exclamation
                }
            }
            '&' => {
                self.chars.next();
                if self.chars.peek() == Some(&'&') {
                    self.chars.next();
                    Token::LogicalAND
                } else {
                    Token::Ampersand
                }
            }
            '"' => {
                self.chars.next();
                let mut string = String::new();

                loop {
                    match self.chars.next() {
                        Some('"') => break,
                        Some(c) => string.push(c),
                        Some('\\') => {
                            let escaped = self.chars.next().ok_or("Unterminated escape char")?;
                            let unescaped = match escaped {
                                'n' => '\n',
                                't' => '\t',
                                '0' => '\0',
                                '\\' => '\\',
                                '\'' => '\'',
                                _ => {
                                    return Err(format!("Unknown escape character: \\{}", escaped));
                                }
                            };
                            string.push(unescaped);
                        }
                        None => {
                            return Err(
                                "Не обнаружено закрывающей кавычки при лексинге строки".into()
                            );
                        }
                    }
                }

                Token::StringConstLiteral(string)
            }
            '\'' => {
                self.chars.next();
                let c = self.chars.next().ok_or("Unexpected EOF in char")?;

                let final_char = if c == '\\' {
                    let escaped = self.chars.next().ok_or("Unterminated escape char")?;
                    match escaped {
                        'n' => '\n',
                        't' => '\t',
                        '0' => '\0',
                        '\\' => '\\',
                        '\'' => '\'',
                        _ => return Err(format!("Unknown escape char: \\{}", escaped)),
                    }
                } else {
                    c
                };

                if self.chars.next() != Some('\'') {
                    return Err(
                        "Expected closing single quote '\'' and char must be 1 byte.".into(),
                    );
                }

                Token::Char(final_char as u8)
            }
            c if c.is_alphabetic() || c == '_' => self.read_identifier(),
            c if c.is_ascii_digit() => self.read_number(),
            _ => {
                self.chars.next();
                return self.next_token();
            }
        };
        Ok(Some(token))
    }

    pub fn tokenize_all(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Ok(Some(token)) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}
