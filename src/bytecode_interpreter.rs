use crate::lowerer::OpCode;

pub struct BytecodeInterpreter {
    pc: usize,
    stack: Vec<usize>,
}

impl BytecodeInterpreter {
    pub fn new() -> BytecodeInterpreter {
        BytecodeInterpreter {
            pc: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self, program: &[OpCode]) {
        while self.pc < program.len() {
            self.interpret_op(&program[self.pc]);
            self.pc += 1;
        }
    }

    fn interpret_op(&mut self, opcode: &OpCode) {
        match opcode {
            OpCode::Push(value) => {
                self.stack.push(*value);
            }
            OpCode::Pop => {
                self.stack.pop();
            }
            OpCode::Dup => {
                let a = self.stack.pop().unwrap();
                self.stack.push(a);
                self.stack.push(a);
            }
            OpCode::Swap => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(a);
                self.stack.push(b);
            }
            OpCode::Add => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b + a);
            }
            OpCode::Sub => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b - a);
            }
            OpCode::Mul => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b * a);
            }
            OpCode::Div => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b / a);
            }
            OpCode::Mod => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b % a);
            }
            OpCode::Gt => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b > a { 1 } else { 0 });
            }
            OpCode::GtEq => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b >= a { 1 } else { 0 });
            }
            OpCode::Lt => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b < a { 1 } else { 0 });
            }
            OpCode::LtEq => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b <= a { 1 } else { 0 });
            }
            OpCode::Eq => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b == a { 1 } else { 0 });
            }
            OpCode::Print => {
                println!("{}", self.stack.pop().unwrap());
            }
            OpCode::PrintList => {
                let n = self.stack.pop().unwrap();
                print!("[");
                for i in 0..n {
                    print!("{}", self.stack.pop().unwrap());
                    if i < n - 1 {
                        print!(" ");
                    }
                }
                print!("]");
            }
            _ => todo!(),
        }
    }
}
