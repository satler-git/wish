use thiserror::Error;
use wish_lexer::{ListKind, Span, Token};

#[derive(Debug, Clone)]
pub enum CSTKind {
    Ident(String),
    Symbol(String),

    String(String),
    Int(i128),
    Float(f64),

    True,
    Nil,

    List { kind: ListKind, elements: Vec<CST> },

    Comment,
}

#[derive(Debug, Clone)]
pub enum ASTKind {
    Ident(String),
    Symbol(String),

    String(String),
    Int(i128),
    Float(f64),

    True,
    Nil,

    List { kind: ListKind, elements: Vec<AST> },
}

impl From<CSTKind> for Option<ASTKind> {
    fn from(value: CSTKind) -> Self {
        match value {
            CSTKind::Comment => None,
            k => Some(match k {
                CSTKind::Ident(k) => ASTKind::Ident(k),
                CSTKind::Symbol(k) => ASTKind::Symbol(k),
                CSTKind::String(k) => ASTKind::String(k),
                CSTKind::Int(k) => ASTKind::Int(k),
                CSTKind::Float(k) => ASTKind::Float(k),
                CSTKind::True => ASTKind::True,
                CSTKind::Nil => ASTKind::Nil,
                CSTKind::List { kind, elements } => ASTKind::List {
                    kind,
                    elements: elements
                        .into_iter()
                        .filter_map(<CST as Into<Option<AST>>>::into)
                        .collect(),
                },
                _ => unreachable!(),
            }),
        }
    }
}

impl From<CST> for Option<AST> {
    fn from(CST(span, cstk): CST) -> Self {
        <CSTKind as Into<Option<ASTKind>>>::into(cstk).map(|astk| AST(span, astk))
    }
}

#[derive(Debug, Clone)]
pub struct CST(pub Span, pub CSTKind);
#[derive(Debug, Clone)]
pub struct AST(pub Span, pub ASTKind);

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParsingError {
    #[error("The number of closing brackets and opening ones are not equal.")]
    UnmatchedBracket,
}

impl CST {
    pub fn parse(
        tokens: &mut impl Iterator<Item = (Token, Span)>,
    ) -> Result<Vec<Self>, ParsingError> {
        let (cst, bracket) = CST::parse_count_bracket(tokens)?;

        if bracket != 1 {
            Err(ParsingError::UnmatchedBracket)
        } else {
            Ok(cst)
        }
    }

    fn parse_count_bracket(
        tokens: &mut impl Iterator<Item = (Token, Span)>,
    ) -> Result<(Vec<Self>, i32), ParsingError> {
        let mut cst = vec![];
        let mut bracket = 0;

        while let Some((token, span)) = tokens.next() {
            match token {
                Token::BracketOpen(kind) => {
                    let (elements, brackets) = CST::parse_count_bracket(tokens)?;

                    cst.push(CST(span, CSTKind::List { kind, elements }));
                    bracket += 1 + brackets;
                }

                Token::BracketClose => {
                    bracket -= 1;
                    break;
                }

                others => cst.push(CST(
                    span,
                    match others {
                        Token::BlockComment | Token::LineComment => CSTKind::Comment,
                        Token::Ident(name) => CSTKind::Ident(name),
                        Token::Symbol(name) => CSTKind::Symbol(name),
                        Token::Int(value) => CSTKind::Int(value),
                        Token::Float(value) => CSTKind::Float(value),
                        Token::String(value) => CSTKind::String(value),
                        Token::True => CSTKind::Nil,
                        Token::Nil => CSTKind::Nil,
                        _ => unreachable!(),
                    },
                )),
            }
        }

        Ok((cst, bracket))
    }
}
