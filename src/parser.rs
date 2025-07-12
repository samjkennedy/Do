use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug)]
pub enum OpKind {
    PushInt(i64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Dup,
    Swap,
    Print,
}

#[derive(Debug)]
pub struct Op {
    pub kind: OpKind,
    pub span: Span,
}

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, cursor: 0 }
    }

    pub fn parse_op(&mut self) -> Option<Op> {
        let token = self.tokens.get(self.cursor)?;
        let token_span = &token.span;

        self.cursor += 1;

        match token.kind {
            TokenKind::IntLiteral(value) => Some(Op {
                kind: OpKind::PushInt(value),
                span: token_span.clone(),
            }),
            TokenKind::Plus => Some(Op {
                kind: OpKind::Plus,
                span: token_span.clone(),
            }),
            TokenKind::Minus => Some(Op {
                kind: OpKind::Minus,
                span: token_span.clone(),
            }),
            TokenKind::Star => Some(Op {
                kind: OpKind::Multiply,
                span: token_span.clone(),
            }),
            TokenKind::Slash => Some(Op {
                kind: OpKind::Divide,
                span: token_span.clone(),
            }),
            TokenKind::Percent => Some(Op {
                kind: OpKind::Modulo,
                span: token_span.clone(),
            }),
            TokenKind::DupKeyword => Some(Op {
                kind: OpKind::Dup,
                span: token_span.clone(),
            }),
            TokenKind::SwapKeyword => Some(Op {
                kind: OpKind::Swap,
                span: token_span.clone(),
            }),
            TokenKind::PrintKeyword => Some(Op {
                kind: OpKind::Print,
                span: token_span.clone(),
            }),
            TokenKind::Error(_) | TokenKind::EOF => None,
        }
    }
}