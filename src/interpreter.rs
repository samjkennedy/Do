use crate::parser::{Op, OpKind};
use std::cmp::{Ordering, PartialOrd};
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Debug, Clone)]
enum Value {
    Bool(bool),
    Int(i64),
    List(Vec<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(value) => write!(f, "{}", value),
            Value::Int(value) => write!(f, "{}", value),
            Value::List(values) => write!(
                f,
                "[{}]",
                values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs + rhs),
            _ => unreachable!(),
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs - rhs),
            _ => unreachable!(),
        }
    }
}

impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs * rhs),
            _ => unreachable!(),
        }
    }
}

impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs / rhs),
            _ => unreachable!(),
        }
    }
}

impl Rem for Value {
    type Output = Value;
    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => Value::Int(lhs % rhs),
            _ => unreachable!(),
        }
    }
}

pub struct Interpreter {
    stack: Vec<Value>,
}

impl PartialEq<Self> for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(lhs), Value::Int(rhs)) => lhs == rhs,
            (Value::Bool(lhs), Value::Bool(rhs)) => lhs == rhs,
            _ => todo!(),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Int(lhs), Value::Int(rhs)) => lhs.partial_cmp(rhs),
            (Value::Bool(lhs), Value::Bool(rhs)) => lhs.partial_cmp(rhs),
            _ => todo!(),
        }
    }
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { stack: Vec::new() }
    }

    pub fn interpret(&mut self, ops: &Vec<Op>) {
        for op in ops {
            match &op.kind {
                OpKind::PushBool(value) => self.stack.push(Value::Bool(*value)),
                OpKind::PushInt(value) => self.stack.push(Value::Int(*value)),
                OpKind::PushList(ops) => {
                    let mut values = Vec::new();
                    for op in ops {
                        match op.kind {
                            OpKind::PushInt(value) => values.push(Value::Int(value)),
                            _ => unreachable!(),
                        }
                    }
                    self.stack.push(Value::List(values));
                }
                OpKind::Plus => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(a + b);
                }
                OpKind::Minus => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(a - b);
                }
                OpKind::Multiply => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(a * b);
                }
                OpKind::Divide => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(a / b);
                }
                OpKind::Modulo => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(a % b);
                }
                OpKind::LessThan => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a < b));
                }
                OpKind::LessThanEquals => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a <= b));
                }
                OpKind::GreaterThan => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a > b));
                }
                OpKind::GreaterThanEquals => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a >= b));
                }
                OpKind::Equals => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a == b));
                }
                OpKind::Not => {
                    if let Value::Bool(value) = self.stack.pop().unwrap() {
                        self.stack.push(Value::Bool(!value));
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Dup => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a.clone());
                    self.stack.push(a);
                }
                OpKind::Over => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(a.clone());
                    self.stack.push(b);
                    self.stack.push(a);
                }
                OpKind::Pop => {
                    self.stack.pop().unwrap();
                }
                OpKind::Rot => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    let c = self.stack.pop().unwrap();
                    self.stack.push(b);
                    self.stack.push(a);
                    self.stack.push(c);
                }
                OpKind::Swap => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();
                    self.stack.push(a);
                    self.stack.push(b);
                }
                OpKind::Print => {
                    let a = self.stack.pop().unwrap();
                    println!("{}", a);
                }
            }
        }
    }
}
