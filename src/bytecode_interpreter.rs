use crate::lowerer::ByteCodeInstruction;
use std::collections::HashMap;

pub struct BytecodeInterpreter {
    pc: usize,
    rom: Vec<usize>,
    stack: Vec<usize>,
}

impl BytecodeInterpreter {
    pub fn new() -> BytecodeInterpreter {
        BytecodeInterpreter {
            pc: 0,
            rom: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn interpret(
        &mut self,
        program: &[(String, Vec<ByteCodeInstruction>)],
        constants: &[String],
    ) {
        let mut functions = HashMap::new();

        for (name, function) in program {
            //Store the location of this function for later jumping
            functions.insert(name, self.rom.len());

            for instruction in function {
                for word in instruction.to_binary() {
                    self.rom.push(word);
                }
            }
        }

        while self.pc < self.rom.len() {
            let opcode = self.rom[self.pc];
            let (bytecode_instruction, words_consumed) =
                ByteCodeInstruction::decode(opcode, &self.rom[self.pc + 1..]);

            self.pc += words_consumed;

            self.interpret_op(&bytecode_instruction);
        }
    }

    fn interpret_op(&mut self, opcode: &ByteCodeInstruction) {
        match opcode {
            ByteCodeInstruction::Push(value) => {
                self.stack.push(*value);
            }
            ByteCodeInstruction::Pop => {
                self.stack.pop();
            }
            ByteCodeInstruction::Dup => {
                let a = self.stack.pop().unwrap();
                self.stack.push(a);
                self.stack.push(a);
            }
            ByteCodeInstruction::Swap => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(a);
                self.stack.push(b);
            }
            ByteCodeInstruction::Add => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b + a);
            }
            ByteCodeInstruction::Sub => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b - a);
            }
            ByteCodeInstruction::Mul => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b * a);
            }
            ByteCodeInstruction::Div => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b / a);
            }
            ByteCodeInstruction::Mod => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b % a);
            }
            ByteCodeInstruction::Gt => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b > a { 1 } else { 0 });
            }
            ByteCodeInstruction::GtEq => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b >= a { 1 } else { 0 });
            }
            ByteCodeInstruction::Lt => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b < a { 1 } else { 0 });
            }
            ByteCodeInstruction::LtEq => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b <= a { 1 } else { 0 });
            }
            ByteCodeInstruction::Eq => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(if b == a { 1 } else { 0 });
            }
            ByteCodeInstruction::Print => {
                println!("{}", self.stack.pop().unwrap());
            }
            ByteCodeInstruction::PrintList => {
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
