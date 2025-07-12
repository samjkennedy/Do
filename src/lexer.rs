use crate::diagnostic::Diagnostic;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    IntLiteral(i64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    OpenSquare,
    CloseSquare,
    DupKeyword,
    SwapKeyword,
    PrintKeyword,
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
        self.skip_whitespace();

        let token = match self.peek() {
            Some(c) => match c {
                '+' => self.lex_token(c, TokenKind::Plus),
                '-' => self.lex_token(c, TokenKind::Minus),
                '*' => self.lex_token(c, TokenKind::Star),
                '/' => self.lex_token(c, TokenKind::Slash),
                '%' => self.lex_token(c, TokenKind::Percent),
                '[' => self.lex_token(c, TokenKind::OpenSquare),
                ']' => self.lex_token(c, TokenKind::CloseSquare),
                x if x.is_ascii_digit() => self.lex_number(),
                x if x.is_alphabetic() || x == '_' => self.lex_keyword(),
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

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.cursor += 1;
            } else {
                break;
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
            if c.is_alphanumeric() || c == '_' {
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
            "swap" => Token {
                kind: TokenKind::SwapKeyword,
                span: Span { offset, length },
            },
            "print" => Token {
                kind: TokenKind::PrintKeyword,
                span: Span { offset, length },
            },
            &_ => Token {
                kind: TokenKind::Error(keyword.to_string()),
                span: Span { offset, length },
            },
        }
    }
}
