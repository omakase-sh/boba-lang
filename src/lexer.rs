use logos::Logos;
use std::fmt;
use std::ops::Range;

/// Token types for the Boba language
#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    #[token("fun")]
    Fun,
    
    #[token("if")]
    If,
    
    #[token("elseif")]
    ElseIf,
    
    #[token("else")]
    Else,
    
    #[token("loop")]
    Loop,
    
    #[token("till")]
    Till,
    
    #[token("continue")]
    Continue,
    
    #[token("break")]
    Break,
    
    #[token("return")]
    Return,
    
    #[token("is")]
    Is,
    
    #[token("not")]
    NotKeyword,
    
    #[token("true")]
    True,
    
    #[token("false")]
    False,
    
    #[token("null")]
    Null,
    
    #[token("output")]
    Output,
    
    #[token("outputf")]
    OutputF,
    
    #[token("output&")]
    OutputAddr,
    
    #[token("input")]
    Input,
    
    #[token("inputf")]
    InputF,
    
    // Types
    #[token("int")]
    IntType,
    
    #[token("float")]
    FloatType,
    
    #[token("string")]
    StringType,
    
    #[token("bool")]
    BoolType,
    
    // Literals
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse().ok())]
    IntLiteral(i64),
    
    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().parse().ok())]
    FloatLiteral(f64),
    
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let slice = lex.slice();
        // Remove the quotes and handle escape sequences
        Some(slice[1..slice.len()-1].to_string())
    })]
    StringLiteral(String),
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    // Operators
    #[token("+")]
    Plus,
    
    #[token("-")]
    Minus,
    
    #[token("*")]
    Star,
    
    #[token("/")]
    Slash,
    
    #[token("%")]
    Percent,
    
    #[token("=")]
    Equals,
    
    #[token("==")]
    DoubleEquals,
    
    #[token("!=")]
    NotEquals,
    
    #[token("<")]
    LessThan,
    
    #[token("<=")]
    LessThanEquals,
    
    #[token(">")]
    GreaterThan,
    
    #[token(">=")]
    GreaterThanEquals,
    
    #[token("&&")]
    And,
    
    #[token("||")]
    Or,
    
    #[token("!")]
    Not,
    
    #[token(".")]
    Dot,
    
    #[token("...")]
    Ellipsis,
    
    // Delimiters
    #[token("(")]
    LParen,
    
    #[token(")")]
    RParen,
    
    #[token("{")]
    LBrace,
    
    #[token("}")]
    RBrace,
    
    #[token("[")]
    LBracket,
    
    #[token("]")]
    RBracket,
    
    #[token(":")]
    Colon,
    
    #[token(",")]
    Comma,
    
    #[token(";")]
    Semicolon,
    
    // Comments (ignored)
    #[regex(r"#[^\n]*", logos::skip)]
    Comment,
    
    #[regex(r"###[^#]*###", logos::skip)]
    MultilineComment,
    
    // Whitespace (ignored)
    #[regex(r"[ \t\n\r]+", logos::skip)]
    Whitespace,
    
    // Error
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::IntLiteral(n) => write!(f, "{}", n),
            Token::FloatLiteral(n) => write!(f, "{}", n),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::Identifier(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenWithSpan {
    pub token: Token,
    pub span: Range<usize>,
}

pub fn tokenize(source: &str) -> Result<Vec<TokenWithSpan>, String> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    
    while let Some(token) = lexer.next() {
        match token {
            Ok(token) => {
                let span = lexer.span();
                tokens.push(TokenWithSpan { token, span });
            }
            Err(_) => {
                let span = lexer.span();
                let line_info = get_line_info(source, span.start);
                return Err(format!(
                    "Lexical error at line {}, column {}: invalid token '{}'",
                    line_info.line,
                    line_info.column,
                    &source[span.clone()]
                ));
            }
        }
    }
    
    Ok(tokens)
}

struct LineInfo {
    line: usize,
    column: usize,
}

fn get_line_info(source: &str, pos: usize) -> LineInfo {
    let mut line = 1;
    let mut column = 1;
    
    for (i, c) in source.char_indices() {
        if i >= pos {
            break;
        }
        
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    
    LineInfo { line, column }
}
