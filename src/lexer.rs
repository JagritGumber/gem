use crate::error::LexError;
use crate::token::Token;

pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Ok(None);
        }

        let ch = self.current_char();

        match ch {
            '(' => {
                self.advance();
                Ok(Some(Token::LParen))
            }
            ')' => {
                self.advance();
                Ok(Some(Token::RParen))
            }
            '{' => {
                self.advance();
                Ok(Some(Token::LBrace))
            }
            '}' => {
                self.advance();
                Ok(Some(Token::RBrace))
            }
            ',' => {
                self.advance();
                Ok(Some(Token::Comma))
            }
            ':' => {
                self.advance();
                Ok(Some(Token::Colon))
            }
            '.' => {
                self.advance();
                Ok(Some(Token::Dot))
            }
            ';' => {
                self.advance();
                Ok(Some(Token::Semi))
            }
            '+' => {
                self.advance();
                Ok(Some(Token::Plus))
            }
            '-' => {
                self.advance();
                Ok(Some(Token::Minus))
            }
            '*' => {
                self.advance();
                Ok(Some(Token::Multiply))
            }
            '/' => {
                self.advance();
                Ok(Some(Token::Divide))
            }
            '=' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::EqEq))
                } else {
                    self.advance();
                    Ok(Some(Token::Eq))
                }
            }
            '!' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::NotEq))
                } else {
                    self.advance();
                    Ok(Some(Token::Not))
                }
            }
            '<' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::LessEq))
                } else {
                    self.advance();
                    Ok(Some(Token::Less))
                }
            }
            '>' => {
                if self.peek_char() == Some('=') {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::GreaterEq))
                } else {
                    self.advance();
                    Ok(Some(Token::Greater))
                }
            }
            '&' => {
                if self.peek_char() == Some('&') {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::And))
                } else {
                    Err(LexError {
                        message: "Expected '&&'".to_string(),
                        line: self.line,
                        column: self.column,
                    })
                }
            }
            '|' => {
                if self.peek_char() == Some('|') {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::Or))
                } else {
                    Err(LexError {
                        message: "Expected '||'".to_string(),
                        line: self.line,
                        column: self.column,
                    })
                }
            }
            '"' => self.read_string(),
            _ if ch.is_ascii_digit() => self.read_number(),
            _ if ch.is_ascii_alphabetic() || ch == '_' => self.read_identifier(),
            _ => Err(LexError {
                message: format!("Unexpected character: '{}'", ch),
                line: self.line,
                column: self.column,
            }),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.advance();
            } else {
                break;
            }
        }
    }

    fn current_char(&self) -> char {
        self.input.chars().nth(self.position).unwrap_or('\0')
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
            self.column += 1;
        }
    }

    fn read_string(&mut self) -> Result<Option<Token>, LexError> {
        self.advance(); // Skip opening quote
        let mut value = String::new();

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == '"' {
                self.advance(); // Skip closing quote
                return Ok(Some(Token::String(value)));
            } else if ch == '\\' {
                self.advance();
                if self.position >= self.input.len() {
                    return Err(LexError {
                        message: "Unterminated string literal".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                }
                let escaped = self.current_char();
                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    _ => {
                        value.push('\\');
                        value.push(escaped);
                    }
                }
                self.advance();
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(LexError {
            message: "Unterminated string literal".to_string(),
            line: self.line,
            column: self.column,
        })
    }

    fn read_number(&mut self) -> Result<Option<Token>, LexError> {
        let mut value = String::new();
        let mut is_float = false;

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                // Check if next character is a digit to avoid conflicts with method calls
                if let Some(next_ch) = self.peek_char() {
                    if next_ch.is_ascii_digit() {
                        is_float = true;
                        value.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if is_float {
            match value.parse::<f64>() {
                Ok(num) => Ok(Some(Token::Float(num))),
                Err(_) => Err(LexError {
                    message: format!("Invalid float: {}", value),
                    line: self.line,
                    column: self.column,
                }),
            }
        } else {
            match value.parse::<i64>() {
                Ok(num) => Ok(Some(Token::Integer(num))),
                Err(_) => Err(LexError {
                    message: format!("Invalid integer: {}", value),
                    line: self.line,
                    column: self.column,
                }),
            }
        }
    }

    fn read_identifier(&mut self) -> Result<Option<Token>, LexError> {
        let mut value = String::new();

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let token = match value.as_str() {
            "fun" => Token::Fun,
            "if" => Token::If,
            "else" => Token::Else,
            "let" => Token::Let,
            "takethis" => Token::Takethis,
            "no" => Token::No,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            _ => Token::Ident(value),
        };

        Ok(Some(token))
    }
}
