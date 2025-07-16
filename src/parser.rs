use crate::diagnostic::Diagnostic;
use crate::lexer::{Span, Token, TokenKind};
use std::fmt::Display;

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
    If,
    Choice,
}

#[derive(Debug, Clone)]
pub struct Op {
    pub kind: OpKind,
    pub span: Span,
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            OpKind::PushBool(value) => write!(f, "{}", value),
            OpKind::PushInt(value) => write!(f, "{}", value),
            OpKind::PushList(list) => {
                write!(f, "[")?;
                for (i, op) in list.iter().enumerate() {
                    write!(f, "{}", op)?;
                    if i < list.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, "]")
            }
            OpKind::PushBlock(block) => {
                write!(f, "(")?;
                for (i, op) in block.iter().enumerate() {
                    write!(f, "{}", op)?;
                    if i < block.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")
            }
            OpKind::Plus => write!(f, "+"),
            OpKind::Minus => write!(f, "-"),
            OpKind::Multiply => write!(f, "*"),
            OpKind::Divide => write!(f, "/"),
            OpKind::Modulo => write!(f, "%"),
            OpKind::LessThan => write!(f, "<"),
            OpKind::LessThanEquals => write!(f, "<="),
            OpKind::GreaterThan => write!(f, ">"),
            OpKind::GreaterThanEquals => write!(f, ">="),
            OpKind::Equals => write!(f, "="),
            OpKind::Not => write!(f, "not"),
            OpKind::And => write!(f, "and"),
            OpKind::Or => write!(f, "or"),
            OpKind::Identity => write!(f, "."),
            OpKind::Over => write!(f, "over"),
            OpKind::Pop => write!(f, "pop"),
            OpKind::Rot => write!(f, "rot"),
            OpKind::Swap => write!(f, "swap"),
            OpKind::Dup => write!(f, "dup"),
            OpKind::Print => write!(f, "print"),
            OpKind::Concat => write!(f, "concat"),
            OpKind::Do => write!(f, "do"),
            OpKind::Filter => write!(f, "filter"),
            OpKind::Fold => write!(f, "fold"),
            OpKind::Foreach => write!(f, "foreach"),
            OpKind::Len => write!(f, "len"),
            OpKind::Map => write!(f, "map"),
            OpKind::DumpStack => write!(f, "???"),
            OpKind::DefineFunction { identifier, body } => {
                if let TokenKind::Identifier(name) = &identifier.kind {
                    write!(f, "fn {} {}", name, body)
                } else {
                    unreachable!()
                }
            }
            OpKind::Call(name) => write!(f, "{}", name),
            OpKind::If => write!(f, "if"),
            OpKind::Choice => write!(f, "choice"),
        }
    }
}

pub struct Parser {
    cursor: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            cursor: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn parse(&mut self, tokens: &[Token]) -> Vec<Op> {
        let mut ops: Vec<Op> = vec![];
        while let Some(op) = self.parse_op(tokens) {
            ops.push(op);
        }
        ops
    }

    pub fn parse_op(&mut self, tokens: &[Token]) -> Option<Op> {
        let token = tokens.get(self.cursor)?.clone();

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
            TokenKind::OpenParenthesis => self.parse_block(&token, tokens),
            TokenKind::CloseParenthesis => {
                self.diagnostics.push(Diagnostic::report_error(
                    "unexpected token ')'".to_string(),
                    token.span,
                ));
                None
            }
            TokenKind::OpenSquare => {
                let mut elements = Vec::new();

                while self.cursor < tokens.len()
                    && tokens[self.cursor].kind != TokenKind::CloseSquare
                {
                    elements.push(self.parse_op(tokens)?);
                }

                if self.cursor >= tokens.len() {
                    self.diagnostics.push(Diagnostic::report_error(
                        "List missing closing ']'".to_string(),
                        Span::from_to(
                            token.span,
                            elements.last().map(|op| op.span).unwrap_or(token.span),
                        ),
                    ));
                    return None;
                }

                let close_square = tokens[self.cursor].clone();
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
                let identifier = self.expect_identifier(tokens, token.span)?;
                let open_parenthesis =
                    self.expect_token(&TokenKind::OpenParenthesis, tokens, token.span)?;
                let body = self.parse_block(&open_parenthesis, tokens)?;

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
            TokenKind::IfKeyword => Some(Op {
                kind: OpKind::If,
                span: token.span,
            }),
            TokenKind::ChoiceKeyword => Some(Op {
                kind: OpKind::Choice,
                span: token.span,
            }),
            TokenKind::Error(_) => None,
        }
    }

    fn parse_block(&mut self, open_paren: &Token, tokens: &[Token]) -> Option<Op> {
        let mut ops = Vec::new();

        while self.cursor < tokens.len() && tokens[self.cursor].kind != TokenKind::CloseParenthesis
        {
            ops.push(self.parse_op(tokens)?);
        }

        if self.cursor >= tokens.len() {
            self.diagnostics.push(Diagnostic::report_error(
                "Block missing closing ')'".to_string(),
                Span::from_to(
                    open_paren.span,
                    ops.last().map(|op| op.span).unwrap_or(open_paren.span),
                ),
            ));
            return None;
        }

        let close_paren = tokens[self.cursor].clone();
        self.cursor += 1; //skip closing paren

        Some(Op {
            kind: OpKind::PushBlock(ops),
            span: Span::from_to(open_paren.span, close_paren.span),
        })
    }

    fn expect_identifier(&mut self, tokens: &[Token], span: Span) -> Option<Token> {
        match tokens.get(self.cursor) {
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
    fn expect_token(
        &mut self,
        expected: &TokenKind,
        tokens: &[Token],
        span: Span,
    ) -> Option<Token> {
        match tokens.get(self.cursor) {
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
