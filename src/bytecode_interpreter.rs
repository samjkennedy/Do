use crate::lowerer::{ByteCodeInstruction, StackFrame};
use std::collections::HashMap;

pub struct BytecodeInterpreter {
    pc: usize,
    rom: Vec<usize>,
    stack: Vec<usize>,
    heap: Vec<usize>,
    locals: Vec<usize>,
    labels: Vec<usize>,
    rsp: usize,
}

impl BytecodeInterpreter {
    pub fn new() -> BytecodeInterpreter {
        let locals: Vec<usize> = vec![0; 8]; //probably not correct, we'll be using rbp for locals
        BytecodeInterpreter {
            pc: 0,
            rom: Vec::new(),
            stack: Vec::new(),
            heap: Vec::new(),
            locals,
            labels: Vec::new(),
            rsp: 0,
        }
    }

    pub fn interpret(&mut self, program: &[(String, StackFrame)], constants: &[String]) {
        let mut functions = HashMap::new();

        for (name, function) in program {
            //Store the location of this function for later jumping
            functions.insert(name, self.rom.len());

            if name == "main" {
                self.pc = self.rom.len();
            }

            for instruction in &function.instructions {
                if let ByteCodeInstruction::Label(label) = instruction {
                    if label >= &self.labels.len() {
                        self.labels.extend(vec![0; label - self.labels.len() + 1]);
                    }
                    self.labels[*label] = self.rom.len();
                }
                for word in &instruction.clone().to_binary() {
                    self.rom.push(*word);
                }
            }
        }

        while self.pc < self.rom.len() {
            let opcode = self.rom[self.pc];
            let (bytecode_instruction, words_consumed) =
                ByteCodeInstruction::decode(opcode, &self.rom[self.pc + 1..]);

            self.pc += words_consumed;

            self.interpret_op(&bytecode_instruction, constants, &functions);
        }
    }

    fn interpret_op(
        &mut self,
        opcode: &ByteCodeInstruction,
        constants: &[String],
        functions: &HashMap<&String, usize>,
    ) {
        // println!(">pc: {}, op: {:?}", self.pc, opcode);
        match opcode {
            ByteCodeInstruction::Push(value) => {
                self.stack.push(*value);
            }
            ByteCodeInstruction::NewList => {
                let length = self.stack.pop().unwrap();

                let ptr = self.alloc(length + 1);
                self.heap[ptr] = length;

                for i in 0..length {
                    let el = self.stack.pop().unwrap();
                    self.heap[ptr + 1 + i] = el;
                }
                self.stack.push(ptr);
            }
            ByteCodeInstruction::PushBlock { index } => {
                self.stack.push(*index);
            }
            ByteCodeInstruction::ListLen => {
                let ptr = self.stack.pop().unwrap();
                let len = self.heap[ptr];
                self.stack.push(len);
            }
            ByteCodeInstruction::ListGet => {
                let index = self.stack.pop().unwrap();
                let ptr = self.stack.pop().unwrap();
                let element = self.heap[ptr + index + 1];
                self.stack.push(element);
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
            ByteCodeInstruction::Over => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(b);
            }
            ByteCodeInstruction::Rot => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                let c = self.stack.pop().unwrap();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(c);
            }
            ByteCodeInstruction::Inc => {
                let a = self.stack.pop().unwrap();
                self.stack.push(a + 1);
            }
            ByteCodeInstruction::Dec => {
                let a = self.stack.pop().unwrap();
                self.stack.push(a - 1);
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
            ByteCodeInstruction::PrintBool => {
                let b = self.stack.pop().unwrap();
                println!("{}", if b > 0 { "true" } else { "false" });
            }
            ByteCodeInstruction::PrintList => {
                let ptr = self.stack.pop().unwrap();

                let len = self.heap[ptr];

                print!("[");
                for i in 0..len {
                    print!("{}", self.heap[ptr + 1 + i]);
                    if i < len - 1 {
                        print!(" ");
                    }
                }
                println!("]");
            }
            ByteCodeInstruction::Call | ByteCodeInstruction::CallNamed(_) => {
                let func = self.stack.pop().unwrap();
                let name = &constants[func];
                let addr = functions.get(name).unwrap();

                self.rsp = self.pc;

                self.pc = *addr;
            }
            ByteCodeInstruction::Return => {
                self.pc = self.rsp;
                self.rsp = 0;
            }
            ByteCodeInstruction::Store { index } => {
                if self.locals.len() <= *index {
                    self.locals.insert(*index, self.stack.pop().unwrap())
                } else {
                    self.locals[*index] = self.stack.pop().unwrap();
                }
            }
            ByteCodeInstruction::Load { index } => {
                self.stack.push(self.locals[*index]);
            }
            ByteCodeInstruction::Label(_) => {}
            ByteCodeInstruction::JumpIfFalse { label } => {
                let cond = self.stack.pop().unwrap();
                if cond == 0 {
                    self.pc = self.labels[*label];
                }
            }
            ByteCodeInstruction::Jump { label } => {
                self.pc = self.labels[*label];
            }
        }
        // println!("(=) {:?}", self.stack);
        // println!("(^) {:?}", self.heap);
        // println!("(*) {:?}", self.locals);
    }

    fn alloc(&mut self, size: usize) -> usize {
        let index = self.heap.len();
        for _i in 0..size {
            self.heap.push(0);
        }
        index
    }
}
