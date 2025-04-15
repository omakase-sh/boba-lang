use crate::ast::{BinaryOperator, Expr, FunctionDef, Program};
use crate::lexer::{Token, TokenWithSpan};
use crate::types::Type;
use std::collections::HashMap;

pub struct Parser {
    tokens: Vec<TokenWithSpan>,
    current: usize,
}

pub fn parse(tokens: Vec<TokenWithSpan>) -> Result<Program, String> {
    let mut parser = Parser {
        tokens,
        current: 0,
    };
    
    parser.parse_program()
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = HashMap::new();
        let mut main_block = Vec::new();
        
        while !self.is_at_end() {
            if self.match_token(&Token::Fun) {
                let func_def = self.parse_function_declaration()?;
                functions.insert(func_def.name.clone(), func_def);
            } else {
                let expr = self.parse_expression()?;
                main_block.push(expr);
            }
        }
        
        Ok(Program {
            functions,
            main_block,
        })
    }
    
    fn parse_function_declaration(&mut self) -> Result<FunctionDef, String> {
        // Parse function name
        let name = if let Some(Token::Identifier(name)) = self.current_token_type() {
            let name_str = name.clone();
            self.advance();
            name_str
        } else {
            return Err("Expected function name after 'fun' keyword".to_string());
        };
        
        // Parse parameters
        self.consume(&Token::LParen, "Expected '(' after function name")?;
        
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let param_name = if let Some(Token::Identifier(name)) = self.current_token_type() {
                    let name_str = name.clone();
                    self.advance();
                    name_str
                } else {
                    return Err("Expected parameter name".to_string());
                };
                
                self.consume(&Token::Colon, "Expected ':' after parameter name")?;
                
                let param_type = self.parse_type()?;
                
                params.push((param_name, param_type));
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RParen, "Expected ')' after parameters")?;
        
        // Parse return types
        let mut return_types = Vec::new();
        if self.match_token(&Token::Colon) {
            loop {
                let return_type = self.parse_type()?;
                return_types.push(return_type);
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        // Parse function body
        self.consume(&Token::LBrace, "Expected '{' before function body")?;
        
        let mut body = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let expr = self.parse_expression()?;
            body.push(expr);
        }
        
        self.consume(&Token::RBrace, "Expected '}' after function body")?;
        
        Ok(FunctionDef {
            name,
            params,
            return_types,
            body,
        })
    }
    
    fn parse_type(&mut self) -> Result<Type, String> {
        match self.current_token_type() {
            Some(Token::IntType) => {
                self.advance();
                Ok(Type::Int)
            }
            Some(Token::FloatType) => {
                self.advance();
                Ok(Type::Float)
            }
            Some(Token::StringType) => {
                self.advance();
                Ok(Type::String)
            }
            Some(Token::BoolType) => {
                self.advance();
                Ok(Type::Bool)
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Type::Null)
            }
            Some(Token::LBracket) => {
                self.advance();
                
                // Check if it's a map type [KeyType:ValueType]
                let elem_type = self.parse_type()?;
                
                if self.match_token(&Token::Colon) {
                    let value_type = self.parse_type()?;
                    self.consume(&Token::RBracket, "Expected ']' after map type")?;
                    Ok(Type::Map(Box::new(elem_type), Box::new(value_type)))
                } else {
                    // It's a list type [ElemType]
                    self.consume(&Token::RBracket, "Expected ']' after list element type")?;
                    Ok(Type::List(Box::new(elem_type)))
                }
            }
            _ => Err(format!(
                "Expected type, got {:?}",
                self.current_token_type()
            )),
        }
    }
    
    // Simplified parse_expression for now
    fn parse_expression(&mut self) -> Result<Expr, String> {
        match self.current_token_type() {
            Some(Token::IntLiteral(n)) => {
                let value = *n;
                self.advance();
                Ok(Expr::IntLiteral(value))
            }
            Some(Token::FloatLiteral(n)) => {
                let value = *n;
                self.advance();
                Ok(Expr::FloatLiteral(value))
            }
            Some(Token::StringLiteral(s)) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::StringLiteral(value))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expr::BoolLiteral(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr::BoolLiteral(false))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Expr::NullLiteral)
            }
            Some(Token::IntType) | Some(Token::FloatType) | Some(Token::StringType) | Some(Token::BoolType) => {
                // Parse type conversion function: int(x), float(x), string(x), bool(x)
                let type_token = self.current_token_type().unwrap().clone();
                self.advance();
                
                self.consume(&Token::LParen, &format!("Expected '(' after type name"))?;
                
                let expr = self.parse_expression()?;
                
                self.consume(&Token::RParen, &format!("Expected ')' after expression"))?;
                
                let target_type = match type_token {
                    Token::IntType => Type::Int,
                    Token::FloatType => Type::Float,
                    Token::StringType => Type::String,
                    Token::BoolType => Type::Bool,
                    _ => unreachable!(),
                };
                
                Ok(Expr::TypeConversion {
                    expr: Box::new(expr),
                    target_type,
                })
            }
            Some(Token::Identifier(name)) => {
                let id = name.clone();
                self.advance();
                
                // Check if it's a variable declaration
                if self.match_token(&Token::Equals) {
                    let value = self.parse_expression()?;
                    Ok(Expr::VarDeclaration(id, Box::new(value)))
                } else if self.match_token(&Token::LParen) {
                    // Function call
                    let mut args = Vec::new();
                    
                    if !self.check(&Token::RParen) {
                        loop {
                            let arg = self.parse_expression()?;
                            args.push(arg);
                            
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(&Token::RParen, "Expected ')' after function arguments")?;
                    
                    Ok(Expr::FunctionCall { name: id, args })
                } else {
                    // Variable reference
                    Ok(Expr::Identifier(id))
                }
            }
            Some(Token::Output) => {
                self.advance();
                self.consume(&Token::LParen, "Expected '(' after 'output'")?;
                
                let mut args = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        let arg = self.parse_expression()?;
                        args.push(arg);
                        
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(&Token::RParen, "Expected ')' after output arguments")?;
                
                Ok(Expr::Output(args))
            }
            Some(Token::OutputF) => {
                self.advance();
                self.consume(&Token::LParen, "Expected '(' after 'outputf'")?;
                
                let format_string = self.parse_expression()?;
                
                self.consume(&Token::RParen, "Expected ')' after outputf argument")?;
                
                Ok(Expr::OutputFormatted(Box::new(format_string)))
            }
            Some(Token::Return) => {
                self.advance();
                
                let mut values = Vec::new();
                if !self.check(&Token::RBrace) && !self.is_at_end() {
                    loop {
                        let value = self.parse_expression()?;
                        values.push(value);
                        
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                
                Ok(Expr::Return(values))
            }
            _ => Err(format!(
                "Unexpected token: {:?}",
                self.current_token_type()
            )),
        }
    }
    
    // Helper methods for the parser
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    
    fn current_token_type(&self) -> Option<&Token> {
        self.tokens.get(self.current).map(|t| &t.token)
    }
    
    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1).map(|t| &t.token)
    }
    
    fn check(&self, token_type: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        
        match (self.current_token_type(), token_type) {
            (Some(Token::Identifier(_)), Token::Identifier(_)) => true,
            (Some(Token::IntLiteral(_)), Token::IntLiteral(_)) => true,
            (Some(Token::FloatLiteral(_)), Token::FloatLiteral(_)) => true,
            (Some(Token::StringLiteral(_)), Token::StringLiteral(_)) => true,
            (Some(a), b) => std::mem::discriminant(a) == std::mem::discriminant(b),
            _ => false,
        }
    }
    
    fn match_token(&mut self, token_type: &Token) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }
        false
    }
    
    fn consume(&mut self, token_type: &Token, error_message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance().unwrap())
        } else {
            Err(format!(
                "{}: expected {:?}, got {:?}",
                error_message,
                token_type,
                self.current_token_type()
            ))
        }
    }
}
