use std::{
    num::{ParseFloatError, ParseIntError},
    ops::Range,
};

use logos::{Lexer, Logos};
pub use thiserror::Error;

// TODO: Common lispっぽい構文にするかSchemeっぽいのにするか
pub type Span = Range<usize>;

#[derive(Debug, Default, PartialEq, Eq, Clone, Error)]
pub enum LexingError {
    // TODO: richer error output
    #[error("{0}")]
    ParseInt(ParseIntError),
    #[error("{0}")]
    ParseFloat(ParseFloatError),
    #[default]
    #[error("Unexpected error caught in lexing")]
    Other,
}

impl From<ParseIntError> for LexingError {
    fn from(err: ParseIntError) -> Self {
        LexingError::ParseInt(err)
    }
}
impl From<ParseFloatError> for LexingError {
    fn from(err: ParseFloatError) -> Self {
        LexingError::ParseFloat(err)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKind {
    Normal,
    Data,
}

fn listkind(lex: &mut Lexer<Token>) -> ListKind {
    if lex.slice().len() > 1 {
        match lex.slice().chars().nth(0).unwrap() {
            '\'' => ListKind::Data,
            _ => unimplemented!("There is no such a kind of list"),
        }
    } else {
        ListKind::Normal
    }
}

#[derive(Debug, Logos)]
#[logos(error = LexingError)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(subpattern ident = "([A-Za-z0-9]|[\\\\/\\-+_?!<>&|#*])+")]
#[logos(subpattern newline = r#"(\n|\r|\f)"#)]
#[logos(subpattern num = "[0-9]")]
pub enum Token {
    /// A line comment, e.g. `// comment`.
    #[regex(r#"//[^\n\r\f]*(?&newline)"#)]
    LineComment,

    /// A block comment, e.g. `/* block comment */`.
    #[regex(r#"/\*.*?\*/"#)]
    BlockComment,

    #[regex("(?&ident)", |lex| lex.slice().to_owned())]
    Ident(String),

    #[token("(')?(", listkind)]
    BracketOpen(ListKind),

    #[token(")")]
    BracketClose,

    // // (quote ..)
    // #[token("'(")]
    // QuoteBracketOpen,

    // 'ident
    #[regex("'(?&ident)", |lex| lex.slice().to_owned())]
    Symbol(String),

    #[regex(r"(\+|-)?(?&num)+", |lex| lex.slice().parse(), priority = 5)]
    Int(i128), // TODO? e+みたいな表記とか_を無視したり

    #[regex(r"(\+|-)?((?&num)+\.(?&num)*)|((?&num)*\.(?&num)+)", |lex| lex.slice().parse(), priority = 5)]
    Float(f64),

    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
    String(String),

    #[token("t", priority = 5)]
    True,

    // nil == '()
    #[token("nil")]
    Nil,
}

impl Token {
    pub fn lex(source: &str) -> Result<Vec<(Token, Span)>, LexingError> {
        let mut lexer = Token::lexer(source);
        let mut tokens = vec![];
        while let Some(token) = lexer.next() {
            tokens.push((token?, lexer.span()));
        }
        Ok(tokens)
    }
}

// TODO: test
