use crate::ast::*;
use crate::token::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, got {:?}", expected, self.current()),
            })
        }
    }

    fn is_uppercase_ident(&self, token: &Token) -> bool {
        match token {
            Token::Ident(name) => name.chars().next().map_or(false, |c| c.is_uppercase()),
            _ => false,
        }
    }

    fn is_lowercase_ident(&self, token: &Token) -> bool {
        match token {
            Token::Ident(name) => name
                .chars()
                .next()
                .map_or(false, |c| c.is_lowercase() || c == '_'),
            _ => false,
        }
    }

    /// Parse a scene file: expect one root GemDecl
    pub fn parse_scene(&mut self) -> Result<GemFile, ParseError> {
        let root = self.parse_gem_decl()?;
        Ok(GemFile { root })
    }

    /// Parse GemName: GemType { ... }
    fn parse_gem_decl(&mut self) -> Result<GemDecl, ParseError> {
        // Skip doc comments at the start
        while let Some(Token::DocComment(_)) = self.current() {
            self.advance();
        }

        let name = match self.advance() {
            Some(Token::Ident(n)) if self.is_uppercase_ident(&Token::Ident(n.clone())) => n,
            _ => {
                return Err(ParseError {
                    message: "Expected Gem name (Uppercase identifier)".to_string(),
                });
            }
        };

        self.expect(Token::Colon)?;

        let gem_type = match self.advance() {
            Some(Token::Ident(t)) => t,
            _ => {
                return Err(ParseError {
                    message: "Expected Gem type".to_string(),
                });
            }
        };

        self.expect(Token::LBrace)?;

        let mut properties = Vec::new();
        let mut children = Vec::new();

        while let Some(token) = self.current() {
            if token == &Token::RBrace {
                break;
            }

            // Check if it's a child Gem (Uppercase) or a property (lowercase)
            if self.is_uppercase_ident(token) {
                children.push(self.parse_gem_decl()?);
            } else if self.is_lowercase_ident(token) {
                properties.push(self.parse_property()?);
            } else if token == &Token::Hash {
                // Standalone directive (e.g., link or resource in older style)
                // For now, treat as a special property "link"
                let directive = self.parse_directive()?;
                properties.push(Property {
                    key: "link".to_string(),
                    value: Value::Directive(directive),
                });
            } else if let Token::DocComment(_) = token {
                // skip doc comments inside blocks
                self.advance();
            } else {
                return Err(ParseError {
                    message: format!("Unexpected token in Gem body: {:?}", token),
                });
            }
        }

        self.expect(Token::RBrace)?;

        Ok(GemDecl {
            name,
            gem_type,
            properties,
            children,
        })
    }

    fn parse_property(&mut self) -> Result<Property, ParseError> {
        let key = match self.advance() {
            Some(Token::Ident(k)) => k,
            _ => {
                return Err(ParseError {
                    message: "Expected property key".to_string(),
                });
            }
        };

        self.expect(Token::Colon)?;

        let value = self.parse_value()?;

        Ok(Property { key, value })
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match self.current() {
            Some(Token::Integer(_)) => {
                if let Some(Token::Integer(i)) = self.advance() {
                    Ok(Value::Integer(i))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Float(_)) => {
                if let Some(Token::Float(f)) = self.advance() {
                    Ok(Value::Number(f))
                } else {
                    unreachable!()
                }
            }
            Some(Token::String(s)) => {
                let val = s.clone();
                self.advance();
                Ok(Value::String(val))
            }
            Some(Token::Bool(_)) => {
                if let Some(Token::Bool(b)) = self.advance() {
                    Ok(Value::Bool(b))
                } else {
                    unreachable!()
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let mut elements = Vec::new();
                loop {
                    if let Some(Token::RParen) = self.current() {
                        break;
                    }
                    elements.push(self.parse_value()?);
                    if let Some(Token::Comma) = self.current() {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                Ok(Value::Tuple(elements))
            }
            Some(Token::Hash) => {
                let directive = self.parse_directive()?;
                Ok(Value::Directive(directive))
            }
            Some(Token::Ident(_)) => {
                if let Some(Token::Ident(id)) = self.advance() {
                    Ok(Value::Ident(id))
                } else {
                    unreachable!()
                }
            }
            _ => Err(ParseError {
                message: format!("Expected value, got {:?}", self.current()),
            }),
        }
    }

    fn parse_directive(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(Token::Hash)?;
        let mut segments = Vec::new();
        loop {
            match self.current() {
                Some(Token::Ident(_)) => {
                    if let Some(Token::Ident(seg)) = self.advance() {
                        segments.push(seg);
                        if let Some(Token::Colon) = self.current() {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
        if segments.is_empty() {
            return Err(ParseError {
                message: "Empty directive".to_string(),
            });
        }
        Ok(segments)
    }

    /// Parse a logic file: extend header + events/functions
    pub fn parse_logic(&mut self) -> Result<LogicFile, ParseError> {
        // Skip leading doc comments and capture them
        let mut doc_comment = None;
        while let Some(Token::DocComment(comment)) = self.current() {
            doc_comment = Some(comment.clone());
            self.advance();
        }

        // Parse extend header
        self.expect(Token::Extend)?;
        let extend_type = match self.advance() {
            Some(Token::Ident(t)) => t,
            _ => {
                return Err(ParseError {
                    message: "Expected Gem type after 'extend'".to_string(),
                });
            }
        };

        // Capture doc comment after extend if present
        if let Some(Token::DocComment(comment)) = self.current() {
            doc_comment = Some(comment.clone());
            self.advance();
        }

        let mut events = Vec::new();
        let mut functions = Vec::new();

        while let Some(token) = self.current() {
            match token {
                Token::DocComment(_) => {
                    self.advance();
                }
                Token::Fn => {
                    self.advance();
                    // Check if it's an event handler (on_ready, on_update, etc.) or a regular function
                    if let Some(Token::Ident(name)) = self.current() {
                        if name.starts_with("on_") {
                            // Event handler
                            events.push(self.parse_event_handler()?);
                        } else {
                            // Regular function
                            functions.push(self.parse_function()?);
                        }
                    } else {
                        return Err(ParseError {
                            message: "Expected function or event name after 'fn'".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(ParseError {
                        message: format!("Unexpected token in logic file: {:?}", token),
                    });
                }
            }
        }

        Ok(LogicFile {
            extend_type,
            doc_comment,
            events,
            functions,
        })
    }

    fn parse_event_handler(&mut self) -> Result<Event, ParseError> {
        let name = match self.advance() {
            Some(Token::Ident(n)) => n,
            _ => {
                return Err(ParseError {
                    message: "Expected event name".to_string(),
                });
            }
        };

        let params = if let Some(Token::LParen) = self.current() {
            self.parse_param_list()?
        } else {
            Vec::new()
        };

        let body = self.parse_block()?;

        Ok(Event { name, params, body })
    }

    fn parse_function(&mut self) -> Result<FunctionDecl, ParseError> {
        let name = match self.advance() {
            Some(Token::Ident(n)) => n,
            _ => {
                return Err(ParseError {
                    message: "Expected function name".to_string(),
                });
            }
        };

        let params = self.parse_param_list()?;
        let body = self.parse_block()?;

        Ok(FunctionDecl { name, params, body })
    }

    fn parse_param_list(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        loop {
            if let Some(Token::RParen) = self.current() {
                break;
            }
            match self.advance() {
                Some(Token::Ident(p)) => params.push(p),
                _ => {
                    return Err(ParseError {
                        message: "Expected parameter name".to_string(),
                    });
                }
            }
            if let Some(Token::Comma) = self.current() {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(Token::RParen)?;
        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        while let Some(token) = self.current() {
            if token == &Token::RBrace {
                break;
            }
            statements.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.current() {
            Some(Token::Ident(name)) => {
                let target = name.clone();
                self.advance();
                if let Some(Token::Eq) = self.current() {
                    self.advance();
                    let value = self.parse_expression()?;
                    Ok(Stmt::Assignment { target, value })
                } else {
                    // It's an expression statement (function call)
                    let expr = self.parse_call_or_property(target)?;
                    Ok(Stmt::ExprStmt(expr))
                }
            }
            Some(Token::Spawn) => {
                self.advance();
                self.parse_spawn()
            }
            _ => {
                let expr = self.parse_expression()?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_spawn(&mut self) -> Result<Stmt, ParseError> {
        let gem_type = match self.advance() {
            Some(Token::Ident(t)) => t,
            _ => {
                return Err(ParseError {
                    message: "Expected Gem type after 'spawn'".to_string(),
                });
            }
        };

        self.expect(Token::LBrace)?;
        let mut properties = Vec::new();
        while let Some(token) = self.current() {
            if token == &Token::RBrace {
                break;
            }
            properties.push(self.parse_property()?);
        }
        self.expect(Token::RBrace)?;

        Ok(Stmt::Spawn {
            gem_type,
            properties,
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_logical_and()?;
        while let Some(Token::Or) = self.current() {
            self.advance();
            let right = self.parse_logical_and()?;
            left = Expr::BinaryOp {
                op: BinOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;
        while let Some(Token::And) = self.current() {
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp {
                op: BinOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        while let Some(token) = self.current() {
            let op = match token {
                Token::EqEq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        while let Some(token) = self.current() {
            let op = match token {
                Token::Less => BinOp::Less,
                Token::Greater => BinOp::Greater,
                Token::LessEq => BinOp::LessEq,
                Token::GreaterEq => BinOp::GreaterEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        while let Some(token) = self.current() {
            let op = match token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        while let Some(token) = self.current() {
            let op = match token {
                Token::Multiply => BinOp::Mul,
                Token::Divide => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.current() {
            Some(Token::Not) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Not,
                    expr: Box::new(expr),
                })
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Minus,
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current() {
            Some(Token::Integer(_)) => {
                if let Some(Token::Integer(i)) = self.advance() {
                    Ok(Expr::Integer(i))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Float(_)) => {
                if let Some(Token::Float(f)) = self.advance() {
                    Ok(Expr::Number(f))
                } else {
                    unreachable!()
                }
            }
            Some(Token::String(_)) => {
                if let Some(Token::String(s)) = self.advance() {
                    Ok(Expr::String(s))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Bool(_)) => {
                if let Some(Token::Bool(b)) = self.advance() {
                    Ok(Expr::Bool(b))
                } else {
                    unreachable!()
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let mut elements = Vec::new();
                loop {
                    if let Some(Token::RParen) = self.current() {
                        break;
                    }
                    elements.push(self.parse_expression()?);
                    if let Some(Token::Comma) = self.current() {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(Token::RParen)?;
                Ok(Expr::Tuple(elements))
            }
            Some(Token::Hash) => {
                let directive = self.parse_directive()?;
                Ok(Expr::Directive(directive))
            }
            Some(Token::Ident(_)) => {
                if let Some(Token::Ident(name)) = self.advance() {
                    self.parse_call_or_property(name)
                } else {
                    unreachable!()
                }
            }
            _ => Err(ParseError {
                message: format!("Unexpected token in expression: {:?}", self.current()),
            }),
        }
    }

    fn parse_call_or_property(&mut self, name: String) -> Result<Expr, ParseError> {
        if let Some(Token::LParen) = self.current() {
            // Function call
            self.advance();
            let mut args = Vec::new();
            loop {
                if let Some(Token::RParen) = self.current() {
                    break;
                }
                args.push(self.parse_expression()?);
                if let Some(Token::Comma) = self.current() {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(Token::RParen)?;
            Ok(Expr::Call { name, args })
        } else if let Some(Token::Dot) = self.current() {
            // Property access
            self.advance();
            let property = match self.advance() {
                Some(Token::Ident(p)) => p,
                _ => {
                    return Err(ParseError {
                        message: "Expected property name after '.'".to_string(),
                    });
                }
            };
            let mut expr = Expr::PropertyAccess {
                object: Box::new(Expr::Ident(name)),
                property,
            };
            // Chain property accesses
            while let Some(Token::Dot) = self.current() {
                self.advance();
                let prop = match self.advance() {
                    Some(Token::Ident(p)) => p,
                    _ => {
                        return Err(ParseError {
                            message: "Expected property name after '.'".to_string(),
                        });
                    }
                };
                expr = Expr::PropertyAccess {
                    object: Box::new(expr),
                    property: prop,
                };
            }
            Ok(expr)
        } else {
            Ok(Expr::Ident(name))
        }
    }
}
