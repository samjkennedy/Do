use crate::diagnostic::Diagnostic;
use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug, Clone)]
pub enum OpKind {
    PushBool(bool),
    PushInt(i64),
    PushList(Vec<Op>),
    PushBlock(Vec<Op>),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    Equals,
    Not,
    And,
    Or,
    Identity,
    Over,
    Pop,
    Rot,
    Swap,
    Dup,
    Print,
    Concat,
    Do,
    Filter,
    Fold,
    Foreach,
    Len,
    Map,
    DumpStack,
    DefineFunction { identifier: Token, body: Box<Op> },
    Call(String),
}

#[derive(Debug, Clone)]
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
        let token = self.tokens.get(self.cursor)?.clone();

        self.cursor += 1;

        match token.kind {
            TokenKind::BoolLiteral(value) => Some(Op {
                kind: OpKind::PushBool(value),
                span: token.span,
            }),
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
            TokenKind::OpenAngle => Some(Op {
                kind: OpKind::LessThan,
                span: token.span,
            }),
            TokenKind::OpenAngleEquals => Some(Op {
                kind: OpKind::LessThanEquals,
                span: token.span,
            }),
            TokenKind::CloseAngle => Some(Op {
                kind: OpKind::GreaterThan,
                span: token.span,
            }),
            TokenKind::CloseAngleEquals => Some(Op {
                kind: OpKind::GreaterThanEquals,
                span: token.span,
            }),
            TokenKind::Equals => Some(Op {
                kind: OpKind::Equals,
                span: token.span,
            }),
            TokenKind::Bang => Some(Op {
                kind: OpKind::Not,
                span: token.span,
            }),
            TokenKind::Dot => Some(Op {
                kind: OpKind::Identity,
                span: token.span,
            }),
            TokenKind::AndKeyword => Some(Op {
                kind: OpKind::And,
                span: token.span,
            }),
            TokenKind::OrKeyword => Some(Op {
                kind: OpKind::Or,
                span: token.span,
            }),
            TokenKind::OpenParenthesis => self.parse_block(&token),
            TokenKind::CloseParenthesis => {
                self.diagnostics.push(Diagnostic::report_error(
                    "unexpected token ')'".to_string(),
                    token.span,
                ));
                None
            }
            TokenKind::OpenSquare => {
                let mut elements = Vec::new();

                while self.cursor < self.tokens.len()
                    && self.tokens[self.cursor].kind != TokenKind::CloseSquare
                {
                    elements.push(self.parse_op()?);
                }

                if self.cursor >= self.tokens.len() {
                    self.diagnostics.push(Diagnostic::report_error(
                        "List missing closing ']'".to_string(),
                        Span::from_to(
                            token.span,
                            elements.last().map(|op| op.span).unwrap_or(token.span),
                        ),
                    ));
                    return None;
                }

                let close_square = self.tokens[self.cursor].clone();
                self.cursor += 1; //skip closing paren

                Some(Op {
                    kind: OpKind::PushList(elements),
                    span: Span::from_to(token.span, close_square.span),
                })
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
            TokenKind::OverKeyword => Some(Op {
                kind: OpKind::Over,
                span: token.span,
            }),
            TokenKind::PopKeyword => Some(Op {
                kind: OpKind::Pop,
                span: token.span,
            }),
            TokenKind::SwapKeyword => Some(Op {
                kind: OpKind::Swap,
                span: token.span,
            }),
            TokenKind::RotKeyword => Some(Op {
                kind: OpKind::Rot,
                span: token.span,
            }),
            TokenKind::PrintKeyword => Some(Op {
                kind: OpKind::Print,
                span: token.span,
            }),
            TokenKind::ConcatKeyword => Some(Op {
                kind: OpKind::Concat,
                span: token.span,
            }),
            TokenKind::DoKeyword => Some(Op {
                kind: OpKind::Do,
                span: token.span,
            }),
            TokenKind::FilterKeyword => Some(Op {
                kind: OpKind::Filter,
                span: token.span,
            }),
            TokenKind::FoldKeyword => Some(Op {
                kind: OpKind::Fold,
                span: token.span,
            }),
            TokenKind::ForeachKeyword => Some(Op {
                kind: OpKind::Foreach,
                span: token.span,
            }),
            TokenKind::LenKeyword => Some(Op {
                kind: OpKind::Len,
                span: token.span,
            }),
            TokenKind::MapKeyword => Some(Op {
                kind: OpKind::Map,
                span: token.span,
            }),
            TokenKind::TripleQuestion => Some(Op {
                kind: OpKind::DumpStack,
                span: token.span,
            }),
            TokenKind::FnKeyword => {
                let identifier = self.expect_identifier(token.span)?;
                let open_parenthesis =
                    self.expect_token(&TokenKind::OpenParenthesis, token.span)?;
                let body = self.parse_block(&open_parenthesis)?;

                let span = Span::from_to(identifier.span, body.span);

                Some(Op {
                    kind: OpKind::DefineFunction {
                        identifier,
                        body: Box::new(body),
                    },
                    span,
                })
            }
            TokenKind::Identifier(identifier) => Some(Op {
                kind: OpKind::Call(identifier),
                span: token.span,
            }),
            TokenKind::Error(_) => None,
        }
    }

    fn parse_block(&mut self, token: &Token) -> Option<Op> {
        let mut ops = Vec::new();

        while self.cursor < self.tokens.len()
            && self.tokens[self.cursor].kind != TokenKind::CloseParenthesis
        {
            ops.push(self.parse_op()?);
        }

        if self.cursor >= self.tokens.len() {
            self.diagnostics.push(Diagnostic::report_error(
                "Block missing closing ')'".to_string(),
                Span::from_to(
                    token.span,
                    ops.last().map(|op| op.span).unwrap_or(token.span),
                ),
            ));
            return None;
        }

        let close_paren = self.tokens[self.cursor].clone();
        self.cursor += 1; //skip closing paren

        Some(Op {
            kind: OpKind::PushBlock(ops),
            span: Span::from_to(token.span, close_paren.span),
        })
    }

    fn expect_identifier(&mut self, span: Span) -> Option<Token> {
        match self.tokens.get(self.cursor) {
            Some(token) => match &token.kind {
                TokenKind::Identifier(_) => {
                    self.cursor += 1;
                    Some(token.clone())
                }
                _ => {
                    self.cursor += 1;
                    self.diagnostics.push(Diagnostic::report_error(
                        //TODO: implement display for tokenkind
                        format!("Expected identifier but got `{:?}`", token.kind),
                        span,
                    ));
                    None
                }
            },
            None => {
                self.diagnostics.push(Diagnostic::report_error(
                    "Expected identifier but got nothing".to_string(),
                    span,
                ));
                None
            }
        }
    }
    fn expect_token(&mut self, expected: &TokenKind, span: Span) -> Option<Token> {
        match self.tokens.get(self.cursor) {
            Some(token) => match &token.kind {
                kind if kind == expected => {
                    self.cursor += 1;
                    Some(token.clone())
                }
                _ => {
                    self.cursor += 1;
                    self.diagnostics.push(Diagnostic::report_error(
                        //TODO: implement display for tokenkind
                        format!("Expected '{:?}' but got `{:?}`", expected, token.kind),
                        span,
                    ));
                    None
                }
            },
            None => {
                self.diagnostics.push(Diagnostic::report_error(
                    //TODO: implement display for tokenkind
                    format!("Expected '{:?}' but got nothing", expected),
                    span,
                ));
                None
            }
        }
    }
}
