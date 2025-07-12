use crate::diagnostic::Diagnostic;
use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug)]
pub enum OpKind {
    PushInt(i64),
    PushList(Vec<Op>),
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
    pub diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            cursor: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn parse_op(&mut self) -> Option<Op> {
        let token = self.tokens.get(self.cursor)?;

        self.cursor += 1;

        match token.kind {
            TokenKind::IntLiteral(value) => Some(Op {
                kind: OpKind::PushInt(value),
                span: token.span,
            }),
            TokenKind::Plus => Some(Op {
                kind: OpKind::Plus,
                span: token.span,
            }),
            TokenKind::Minus => Some(Op {
                kind: OpKind::Minus,
                span: token.span,
            }),
            TokenKind::Star => Some(Op {
                kind: OpKind::Multiply,
                span: token.span,
            }),
            TokenKind::Slash => Some(Op {
                kind: OpKind::Divide,
                span: token.span,
            }),
            TokenKind::Percent => Some(Op {
                kind: OpKind::Modulo,
                span: token.span,
            }),
            TokenKind::OpenSquare => {
                let mut ops = Vec::new();

                loop {
                    match self.tokens.get(self.cursor) {
                        Some(next) => {
                            self.cursor += 1;
                            match next.kind {
                                TokenKind::CloseSquare => {
                                    return Some(Op {
                                        kind: OpKind::PushList(ops),
                                        span: Span::from_to(token.span, next.span),
                                    });
                                }
                                TokenKind::IntLiteral(value) => ops.push(Op {
                                    kind: OpKind::PushInt(value),
                                    span: token.span,
                                }),
                                _ => self.diagnostics.push(Diagnostic::report_error(
                                    "Only push operations allowed in lists".to_string(),
                                    next.span,
                                )),
                            }
                        }
                        None => {
                            self.diagnostics.push(Diagnostic::report_error(
                                "List missing closing ']'".to_string(),
                                token.span,
                            ));
                            return None;
                        }
                    }
                }
            }
            TokenKind::CloseSquare => {
                self.diagnostics.push(Diagnostic::report_error(
                    "unexpected token ']'".to_string(),
                    token.span,
                ));
                None
            }
            TokenKind::DupKeyword => Some(Op {
                kind: OpKind::Dup,
                span: token.span,
            }),
            TokenKind::SwapKeyword => Some(Op {
                kind: OpKind::Swap,
                span: token.span,
            }),
            TokenKind::PrintKeyword => Some(Op {
                kind: OpKind::Print,
                span: token.span,
            }),
            TokenKind::Error(_) => None,
        }
    }
}
