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
            '#' => {
                // directive marker
                self.advance();
                return Ok(Some(Token::Hash));
            }
            '/' => {
                match (self.peek_char(), self.peek_n(2)) {
                    (Some('/'), Some('/')) => {
                        // doc comment ///
                        self.advance(); // first /
                        self.advance(); // second /
                        self.advance(); // third /
                        let content = self.collect_line();
                        return Ok(Some(Token::DocComment(content)));
                    }
                    (Some('/'), _) => {
                        self.advance();
                        self.advance();
                        self.skip_rest_of_line();
                        return self.next_token();
                    }
                    (Some('#'), _) => {
                        // multiline comment /# ... #/
                        self.advance(); // '/'
                        self.advance(); // '#'
                        self.skip_multiline_comment()?;
                        return self.next_token();
                    }
                    _ => {
                        self.advance();
                        return Ok(Some(Token::Divide));
                    }
                }
            }
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

    fn peek_n(&self, n: usize) -> Option<char> {
        self.input.chars().nth(self.position + n)
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

        if value == "true" {
            return Ok(Some(Token::Bool(true)));
        }
        if value == "false" {
            return Ok(Some(Token::Bool(false)));
        }

        // logic keywords
        match value.as_str() {
            "on" => return Ok(Some(Token::On)),
            "spawn" => return Ok(Some(Token::Spawn)),
            "extend" => return Ok(Some(Token::Extend)),
            "fn" => return Ok(Some(Token::Fn)),
            _ => {}
        }

        let token = Token::Ident(value);

        Ok(Some(token))
    }

    fn skip_rest_of_line(&mut self) {
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
    fn collect_line(&mut self) -> String {
        let mut value = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch == '\n' {
                break;
            }
            value.push(ch);
            self.advance();
        }
        value.trim().to_string()
    }

    fn skip_multiline_comment(&mut self) -> Result<(), LexError> {
        while self.position < self.input.len() {
            // detect end '#/' sequence
            if self.current_char() == '#' && self.peek_char() == Some('/') {
                self.advance(); // '#'
                self.advance(); // '/'
                return Ok(());
            }
            let ch = self.current_char();
            if ch == '\n' {
                self.position += 1;
                self.line += 1;
                self.column = 1;
            } else {
                self.position += 1;
                self.column += 1;
            }
        }
        Err(LexError {
            message: "Unterminated multiline comment (/# ... #/)".to_string(),
            line: self.line,
            column: self.column,
        })
    }
}
