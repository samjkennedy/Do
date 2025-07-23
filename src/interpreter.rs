use crate::diagnostic::Diagnostic;
use crate::lexer::TokenKind;
use crate::parser::{Op, OpKind};
use std::cmp::{Ordering, PartialOrd};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Int(i64),
    List(Vec<Value>),
    Block(Vec<Op>),
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
            Value::Block(ops) => {
                write!(f, "(")?;
                for (i, op) in ops.iter().enumerate() {
                    write!(f, "{}", op)?;
                    if i + 1 < ops.len() {
                        write!(f, " ")?;
                    }
                }
                write!(f, ")")
            }
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

#[derive(Clone)]
pub struct Interpreter {
    pub stack: Vec<Value>,
    functions: HashMap<String, Vec<Op>>,
    pub diagnostics: Vec<Diagnostic>,
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
        Interpreter {
            stack: Vec::new(),
            functions: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }
    pub fn new_sub(functions: HashMap<String, Vec<Op>>) -> Interpreter {
        Interpreter {
            stack: Vec::new(),
            functions,
            diagnostics: Vec::new(),
        }
    }

    pub fn interpret(&mut self, ops: &Vec<Op>) {
        for op in ops {
            match &op.kind {
                OpKind::PushBool(value) => self.stack.push(Value::Bool(*value)),
                OpKind::PushInt(value) => self.stack.push(Value::Int(*value)),
                OpKind::PushList(ops) => {
                    let mut values = Vec::new();
                    for op in ops {
                        match &op.kind {
                            OpKind::PushInt(value) => values.push(Value::Int(*value)),
                            OpKind::PushBool(value) => values.push(Value::Bool(*value)),
                            OpKind::PushList(elements) => {
                                let mut sub_interpreter =
                                    Interpreter::new_sub(self.functions.clone());
                                sub_interpreter.interpret(elements);
                                values.push(Value::List(sub_interpreter.stack));
                            }
                            OpKind::PushFunction(ops) => {
                                values.push(Value::Block(ops.clone()));
                            }
                            _ => unreachable!(),
                        }
                    }
                    self.stack.push(Value::List(values));
                }
                OpKind::PushFunction(ops) => {
                    self.stack.push(Value::Block(ops.clone()));
                }
                OpKind::Plus => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(b + a);
                }
                OpKind::Minus => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(b - a);
                }
                OpKind::Multiply => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(b * a);
                }
                OpKind::Divide => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(b / a);
                }
                OpKind::Modulo => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(b % a);
                }
                OpKind::LessThan => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(b < a));
                }
                OpKind::LessThanEquals => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(b <= a));
                }
                OpKind::GreaterThan => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(b > a));
                }
                OpKind::GreaterThanEquals => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(b >= a));
                }
                OpKind::Equals => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(b == a));
                }
                OpKind::Not => {
                    if let Value::Bool(value) = self.stack.pop().unwrap() {
                        self.stack.push(Value::Bool(!value));
                    } else {
                        unreachable!()
                    }
                }
                OpKind::And => {
                    if let Value::Bool(a) = self.stack.pop().unwrap() {
                        if let Value::Bool(b) = self.stack.pop().unwrap() {
                            self.stack.push(Value::Bool(a && b));
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Or => {
                    if let Value::Bool(a) = self.stack.pop().unwrap() {
                        if let Value::Bool(b) = self.stack.pop().unwrap() {
                            self.stack.push(Value::Bool(a || b));
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Identity => {
                    //Do I need to even evaluate this?
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a);
                }
                OpKind::Dup => {
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a.clone());
                    self.stack.push(a);
                }
                OpKind::Len => {
                    if let Value::List(values) = self.stack.pop().unwrap() {
                        self.stack.push(Value::Int(values.len() as i64));
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Over => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(b.clone());
                    self.stack.push(a);
                    self.stack.push(b);
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
                OpKind::Concat => {
                    if let Value::List(lhs) = &self.stack.pop().unwrap() {
                        if let Value::List(rhs) = &self.stack.pop().unwrap() {
                            let mut result = Vec::new();
                            result.extend(rhs.clone());
                            result.extend(lhs.clone());
                            self.stack.push(Value::List(result));
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Push => {
                    let value = &self.stack.pop().unwrap();
                    if let Value::List(values) = &self.stack.pop().unwrap() {
                        let mut result = Vec::new();
                        result.extend(values.clone());
                        result.push(value.clone());
                        self.stack.push(Value::List(result));
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Head => {
                    if let Value::List(values) = &self.stack.pop().unwrap() {
                        if values.is_empty() {
                            self.diagnostics.push(Diagnostic::report_error(
                                "Cannot `head` an empty list".to_string(),
                                op.span,
                            ));
                            continue;
                        }
                        self.stack.push(values[0].clone());
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Tail => {
                    if let Value::List(values) = &self.stack.pop().unwrap() {
                        if values.is_empty() {
                            self.stack.push(Value::List(vec![]));
                            continue;
                        }
                        let result = values[1..].to_vec().clone();
                        self.stack.push(Value::List(result));
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Do => {
                    let value = self.stack.pop().unwrap();
                    if let Value::Block(ops) = &value {
                        self.interpret(ops);
                    } else {
                        unreachable!("tried to call `do` on {:?}", value)
                    }
                }
                OpKind::Filter => {
                    if let Value::Block(ops) = &self.stack.pop().unwrap() {
                        if let Value::List(values) = &self.stack.pop().unwrap() {
                            let mut sub_interpreter = Interpreter::new_sub(self.functions.clone());
                            for value in values {
                                sub_interpreter.stack.push(value.clone());
                                sub_interpreter.interpret(ops);
                            }
                            let mut results = Vec::new();
                            for (i, result) in sub_interpreter.stack.iter().enumerate() {
                                if let Value::Bool(result) = result {
                                    if *result {
                                        results.push(values[i].clone());
                                    }
                                } else {
                                    unreachable!()
                                }
                            }
                            self.stack.push(Value::List(results));
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Foreach => {
                    if let Value::Block(ops) = &self.stack.pop().unwrap() {
                        if let Value::List(values) = &self.stack.pop().unwrap() {
                            for value in values {
                                self.stack.push(value.clone());
                                self.interpret(ops);
                            }
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Map => {
                    if let Value::Block(ops) = &self.stack.pop().unwrap() {
                        if let Value::List(values) = &self.stack.pop().unwrap() {
                            let mut sub_interpreter = Interpreter::new_sub(self.functions.clone());
                            for value in values {
                                sub_interpreter.stack.push(value.clone());
                                sub_interpreter.interpret(ops);
                            }
                            self.stack.push(Value::List(sub_interpreter.stack));
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Fold => {
                    let mut acc = self.stack.pop().unwrap();
                    if let Value::Block(ops) = &self.stack.pop().unwrap() {
                        if let Value::List(values) = &self.stack.pop().unwrap() {
                            let mut sub_interpreter = Interpreter::new_sub(self.functions.clone());
                            for value in values {
                                sub_interpreter.stack.push(acc.clone());
                                sub_interpreter.stack.push(value.clone());
                                sub_interpreter.interpret(ops);
                                acc = sub_interpreter.stack.pop().unwrap();
                            }
                            self.stack.push(acc);
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpKind::DumpStack => {}
                OpKind::DefineFunction { identifier, body } => {
                    if let TokenKind::Identifier(name) = &identifier.kind {
                        if let OpKind::PushFunction(ops) = &body.kind {
                            self.functions.insert(name.clone(), ops.clone());
                        } else {
                            unreachable!()
                        }
                    } else {
                        unreachable!()
                    }
                }
                OpKind::Identifier(name) => {
                    let ops = self.functions.get(name).unwrap().clone();
                    self.interpret(&ops);
                }
                OpKind::If { .. } => todo!(),
                OpKind::Binding { .. } => todo!(),
            }
        }
    }
}
