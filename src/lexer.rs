use crate::diagnostic::Diagnostic;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    Identifier(String),
    IntLiteral(i64),
    BoolLiteral(bool),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    OpenParenthesis,
    CloseParenthesis,
    OpenAngle,
    OpenAngleEquals,
    CloseAngle,
    CloseAngleEquals,
    Equals,
    Bang,
    Dot,
    AndKeyword,
    OrKeyword,
    OpenSquare,
    CloseSquare,
    DupKeyword,
    LenKeyword,
    OverKeyword,
    PopKeyword,
    RotKeyword,
    SwapKeyword,
    PrintKeyword,
    ConcatKeyword,
    DoKeyword,
    FilterKeyword,
    FoldKeyword,
    ForeachKeyword,
    MapKeyword,
    TripleQuestion,
    FnKeyword,
    Error(String),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Span {
    pub offset: usize,
    pub length: usize,
}

impl Span {
    pub fn from_to(start: Span, end: Span) -> Self {
        Span {
            offset: start.offset,
            length: end.offset - start.offset + end.length,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer {
    input: String,
    cursor: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        Lexer {
            input,
            cursor: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        self.skip_whitespace_and_comments();

        let token = match self.peek() {
            Some(c) => match c {
                '+' => self.lex_token(c, TokenKind::Plus),
                '-' => self.lex_token(c, TokenKind::Minus),
                '*' => self.lex_token(c, TokenKind::Star),
                '/' => self.lex_token(c, TokenKind::Slash),
                '%' => self.lex_token(c, TokenKind::Percent),
                '(' => self.lex_token(c, TokenKind::OpenParenthesis),
                ')' => self.lex_token(c, TokenKind::CloseParenthesis),
                '<' => self.lex_multichar_token(
                    c,
                    '=',
                    TokenKind::OpenAngleEquals,
                    TokenKind::OpenAngle,
                ),
                '>' => self.lex_multichar_token(
                    c,
                    '=',
                    TokenKind::CloseAngleEquals,
                    TokenKind::CloseAngle,
                ),
                '=' => self.lex_token(c, TokenKind::Equals),
                '!' => self.lex_token(c, TokenKind::Bang),
                '.' => self.lex_token(c, TokenKind::Dot),
                '[' => self.lex_token(c, TokenKind::OpenSquare),
                ']' => self.lex_token(c, TokenKind::CloseSquare),
                x if x.is_ascii_digit() => self.lex_number(),
                x if x.is_alphabetic() || x == '_' || x == '?' => self.lex_keyword(),
                _ => {
                    let error = Token {
                        kind: TokenKind::Error(c.to_string()),
                        span: Span {
                            offset: self.cursor,
                            length: c.len_utf8(),
                        },
                    };

                    self.diagnostics.push(Diagnostic::report_error(
                        format!("Unexpected character `{}`", c),
                        Span {
                            offset: self.cursor,
                            length: c.len_utf8(),
                        },
                    ));
                    self.cursor += 1;

                    return Some(error);
                }
            },
            None => return None,
        };

        Some(token)
    }

    fn lex_multichar_token(
        &mut self,
        c: char,
        next: char,
        if_match: TokenKind,
        if_not_match: TokenKind,
    ) -> Token {
        if self.cursor < self.input.len() {
            self.cursor += 1;
            return match self.peek() {
                Some(c) if c == next => {
                    let token = Token {
                        kind: if_match,
                        span: Span {
                            offset: self.cursor,
                            length: 2,
                        },
                    };
                    self.cursor += 1;
                    token
                }
                _ => {
                    self.cursor -= 1; //backpedal
                    self.lex_token(c, if_not_match)
                }
            };
        }
        self.lex_token(c, if_not_match)
    }

    fn lex_token(&mut self, c: char, kind: TokenKind) -> Token {
        let token = Token {
            kind,
            span: Span {
                offset: self.cursor,
                length: c.len_utf8(),
            },
        };
        self.cursor += 1;
        token
    }

    fn peek(&mut self) -> Option<char> {
        self.input.chars().nth(self.cursor)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            let start = self.cursor;
            self.skip_single_whitespace();
            self.skip_comment();
            if self.cursor == start {
                break;
            }
        }
    }

    fn skip_single_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        if let Some('/') = self.peek() {
            self.cursor += 1;
            if let Some('/') = self.peek() {
                self.cursor += 1;
                while let Some(c) = self.peek() {
                    self.cursor += 1;
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                self.cursor -= 1; //backpedal
            }
        }
    }

    fn lex_number(&mut self) -> Token {
        let offset = self.cursor;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.cursor += 1;
            } else {
                break;
            }
        }

        let number = &self.input[offset..self.cursor].parse::<i64>().unwrap();
        Token {
            kind: TokenKind::IntLiteral(*number),
            span: Span {
                offset,
                length: self.cursor - offset,
            },
        }
    }

    fn lex_keyword(&mut self) -> Token {
        let offset = self.cursor;

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '?' {
                self.cursor += 1;
            } else {
                break;
            }
        }

        let keyword = &self.input[offset..self.cursor];
        let length = self.cursor - offset;

        match keyword {
            "dup" => Token {
                kind: TokenKind::DupKeyword,
                span: Span { offset, length },
            },
            "over" => Token {
                kind: TokenKind::OverKeyword,
                span: Span { offset, length },
            },
            "pop" => Token {
                kind: TokenKind::PopKeyword,
                span: Span { offset, length },
            },
            "rot" => Token {
                kind: TokenKind::RotKeyword,
                span: Span { offset, length },
            },
            "swap" => Token {
                kind: TokenKind::SwapKeyword,
                span: Span { offset, length },
            },
            "print" => Token {
                kind: TokenKind::PrintKeyword,
                span: Span { offset, length },
            },
            "true" => Token {
                kind: TokenKind::BoolLiteral(true),
                span: Span { offset, length },
            },
            "false" => Token {
                kind: TokenKind::BoolLiteral(false),
                span: Span { offset, length },
            },
            "and" => Token {
                kind: TokenKind::AndKeyword,
                span: Span { offset, length },
            },
            "or" => Token {
                kind: TokenKind::OrKeyword,
                span: Span { offset, length },
            },
            "concat" => Token {
                kind: TokenKind::ConcatKeyword,
                span: Span { offset, length },
            },
            "do" => Token {
                kind: TokenKind::DoKeyword,
                span: Span { offset, length },
            },
            "filter" => Token {
                kind: TokenKind::FilterKeyword,
                span: Span { offset, length },
            },
            "fold" => Token {
                kind: TokenKind::FoldKeyword,
                span: Span { offset, length },
            },
            "foreach" => Token {
                kind: TokenKind::ForeachKeyword,
                span: Span { offset, length },
            },
            "len" => Token {
                kind: TokenKind::LenKeyword,
                span: Span { offset, length },
            },
            "map" => Token {
                kind: TokenKind::MapKeyword,
                span: Span { offset, length },
            },
            "???" => Token {
                kind: TokenKind::TripleQuestion,
                span: Span { offset, length },
            },
            "fn" => Token {
                kind: TokenKind::FnKeyword,
                span: Span { offset, length },
            },
            &_ => Token {
                kind: TokenKind::Identifier(keyword.to_string()),
                span: Span { offset, length },
            },
        }
    }
}
