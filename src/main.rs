use std::cmp::PartialEq;

#[derive(PartialEq, Debug, Clone)]
enum TokenKind {
    IntLiteral(i64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    DupKeyword,
    SwapKeyword,
    PrintKeyword,
    Error(String),
    EOF,
}

#[derive(PartialEq, Debug, Clone)]
struct Span {
    offset: usize,
    length: usize,
}

#[derive(PartialEq, Debug, Clone)]
struct Token {
    kind: TokenKind,
    span: Span,
}

struct Lexer {
    input: String,
    cursor: usize,
}

impl Lexer {
    fn new(input: String) -> Lexer {
        Lexer { input, cursor: 0 }
    }

    fn next(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.peek() {
            Some(c) => match c {
                '+' => self.lex_token(c, TokenKind::Plus),
                '-' => self.lex_token(c, TokenKind::Minus),
                '*' => self.lex_token(c, TokenKind::Star),
                '/' => self.lex_token(c, TokenKind::Slash),
                '%' => self.lex_token(c, TokenKind::Percent),
                x if x.is_ascii_digit() => self.lex_number(),
                x if x.is_alphabetic() || x == '_' => self.lex_keyword(),
                _ => Token {
                    kind: TokenKind::Error(c.to_string()),
                    span: Span {
                        offset: self.cursor,
                        length: c.len_utf8(),
                    },
                },
            },
            None => Token {
                kind: TokenKind::EOF,
                span: Span {
                    offset: self.cursor,
                    length: 0,
                },
            },
        };

        token
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

// ===== //

#[derive(Debug)]
enum OpKind {
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
struct Op {
    kind: OpKind,
    span: Span,
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, cursor: 0 }
    }

    fn parse_op(&mut self) -> Option<Op> {
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

// ==== //

#[derive(Debug, Clone)]
enum TypeKind {
    Bool,
    Int,
}

struct TypeChecker {
    type_stack: Vec<TypeKind>,
}

impl TypeChecker {
    fn new() -> TypeChecker {
        TypeChecker {
            type_stack: Vec::new(),
        }
    }

    fn type_check(&mut self, ops: &Vec<Op>) {
        for op in ops {
            match op.kind {
                OpKind::PushInt(_) => self.type_stack.push(TypeKind::Int),
                OpKind::Plus
                | OpKind::Minus
                | OpKind::Multiply
                | OpKind::Divide
                | OpKind::Modulo => match (self.type_stack.pop(), self.type_stack.pop()) {
                    (Some(TypeKind::Int), Some(TypeKind::Int)) => {
                        self.type_stack.push(TypeKind::Int)
                    }
                    (Some(a), Some(b)) => panic!("Expected [Int Int] but got [{:?} {:?}]", a, b),
                    _ => panic!("Expected [Int Int] but stack was empty"),
                },
                OpKind::Dup => {
                    match self.type_stack.pop() {
                        Some(a) => {
                            self.type_stack.push(a.clone());
                            self.type_stack.push(a);
                        },
                        None => panic!("Expected `Any` but stack was empty"),
                    }
                }
                OpKind::Swap => {
                    match self.type_stack.pop() {
                        Some(a) => match self.type_stack.pop() {
                            Some(b) => {
                                self.type_stack.push(a);
                                self.type_stack.push(b);
                            },
                            None => panic!("Expected `Any` but stack was empty"),
                        },
                        None => panic!("Expected `Any` but stack was empty"),
                    }
                }
                OpKind::Print => match self.type_stack.pop() {
                    Some(_) => (),
                    None => panic!("Expected `Any` but stack was empty"),
                },
            }
        }
    }
}

// ===== //

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
}

struct Interpreter {
    stack: Vec<Value>,
}

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter { stack: Vec::new() }
    }

    fn interpret(&mut self, ops: &Vec<Op>) {
        for op in ops {
            match op.kind {
                OpKind::PushInt(value) => self.stack.push(Value::Int(value)),
                OpKind::Plus => {
                    let Value::Int(a) = self.stack.pop().unwrap();
                    let Value::Int(b) = self.stack.pop().unwrap();

                    self.stack.push(Value::Int(a + b));
                }
                OpKind::Minus => {
                    let Value::Int(a) = self.stack.pop().unwrap();
                    let Value::Int(b) = self.stack.pop().unwrap();

                    self.stack.push(Value::Int(a - b));
                }
                OpKind::Multiply => {
                    let Value::Int(a) = self.stack.pop().unwrap();
                    let Value::Int(b) = self.stack.pop().unwrap();

                    self.stack.push(Value::Int(a * b));
                }
                OpKind::Divide => {
                    let Value::Int(a) = self.stack.pop().unwrap();
                    let Value::Int(b) = self.stack.pop().unwrap();

                    self.stack.push(Value::Int(a / b));
                }
                OpKind::Modulo => {
                    let Value::Int(a) = self.stack.pop().unwrap();
                    let Value::Int(b) = self.stack.pop().unwrap();

                    self.stack.push(Value::Int(a % b));
                }
                OpKind::Dup => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a.clone());
                    self.stack.push(a);
                }
                OpKind::Swap => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a);
                    self.stack.push(b);
                }
                OpKind::Print => {
                    let Value::Int(a) = self.stack.pop().unwrap();
                    println!("{}", a);
                }
            }
        }
    }
}

fn main() {
    let filename = "resources/arithmetic.do";
    let input = std::fs::read_to_string(filename).unwrap();

    let mut lexer = Lexer::new(input);

    let mut tokens: Vec<Token> = vec![];

    while let token = lexer.next() {
        if token.kind == TokenKind::EOF {
            break;
        }
        // println!("{:?}", token);
        tokens.push(token);
    }

    let mut parser = Parser::new(tokens);
    let mut ops: Vec<Op> = vec![];
    while let Some(op) = parser.parse_op() {
        // println!("{:?}", op);
        ops.push(op);
    }

    let mut type_checker = TypeChecker::new();
    type_checker.type_check(&ops);

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ops);
}
