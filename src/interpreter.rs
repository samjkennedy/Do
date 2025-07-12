use crate::parser::{Op, OpKind};

#[derive(Debug, Clone)]
enum Value {
    Int(i64),
}

pub struct Interpreter {
    stack: Vec<Value>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter { stack: Vec::new() }
    }

    pub fn interpret(&mut self, ops: &Vec<Op>) {
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
