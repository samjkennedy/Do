use std::cmp::max;
use crate::typechecker::{TypeKind, TypedOp, TypedOpKind};
use std::collections::HashMap;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ByteCodeInstruction {
    //Pushes a literal onto the stack
    Push(usize),
    //Pops a literal from the stack
    Pop,
    //Pops a length from the stack and constructs a list from that many stack elements,
    // pushing the pointer to the list back onto the stack
    NewList,
    //Pops a pointer to a list and pushes the length of the list to the stack
    ListLen,
    //Pops a pointer to a list and an index pushes that element of the list to the stack
    ListGet,
    //Pushes a pointer to the function given by the index onto the stack
    PushBlock { index: usize },
    //Push the local given by the index onto the stack
    Load { index: usize },
    //Pop the index to a local and a value and store the value in the local
    Store { index: usize },
    Dup,
    Over,
    Rot,
    Swap,
    Inc,
    Dec,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Gt,
    Lt,
    GtEq,
    LtEq,
    Eq,
    Print,
    PrintBool,
    PrintList,
    Label(usize),
    //Call a known function by the index in the constant pool
    CallStatic { index: usize },
    //Pops a function pointer from the stack and calls it
    CallDynamic,
    Jump { label: usize },
    JumpIfFalse { label: usize },
    Return,
}

impl ByteCodeInstruction {
    fn get_opcode(&self) -> usize {
        match self {
            ByteCodeInstruction::Push(_) => 0x01,
            ByteCodeInstruction::Pop => 0x02,
            ByteCodeInstruction::NewList => 0x03,
            ByteCodeInstruction::ListLen => 0x04,
            ByteCodeInstruction::ListGet => 0x05,
            // ByteCodeInstruction::ListSet => 0x06,
            ByteCodeInstruction::PushBlock { .. } => 0x07,
            ByteCodeInstruction::Load { .. } => 0x08,
            ByteCodeInstruction::Store { .. } => 0x09,
            ByteCodeInstruction::Dup => 0x0A,
            ByteCodeInstruction::Over => 0x0B,
            ByteCodeInstruction::Rot => 0x0C,
            ByteCodeInstruction::Swap => 0x0D,
            ByteCodeInstruction::Add => 0x0E,
            ByteCodeInstruction::Sub => 0x0F,
            ByteCodeInstruction::Mul => 0x10,
            ByteCodeInstruction::Div => 0x11,
            ByteCodeInstruction::Mod => 0x12,
            ByteCodeInstruction::Gt => 0x13,
            ByteCodeInstruction::Lt => 0x14,
            ByteCodeInstruction::GtEq => 0x15,
            ByteCodeInstruction::LtEq => 0x16,
            ByteCodeInstruction::Eq => 0x17,
            ByteCodeInstruction::Print => 0x18,
            ByteCodeInstruction::PrintList => 0x19,
            ByteCodeInstruction::Label(_) => 0x1A,
            ByteCodeInstruction::CallStatic { .. } => 0x1B,
            ByteCodeInstruction::CallDynamic => 0x1C,
            ByteCodeInstruction::Jump { .. } => 0x1D,
            ByteCodeInstruction::JumpIfFalse { .. } => 0x1E,
            ByteCodeInstruction::Return => 0x1F,
            ByteCodeInstruction::Inc => 0x20,
            ByteCodeInstruction::Dec => 0x21,
            ByteCodeInstruction::PrintBool => 0x22,
        }
    }

    pub fn to_binary(&self) -> Vec<usize> {
        match self {
            ByteCodeInstruction::Push(value) => vec![self.get_opcode(), *value],
            ByteCodeInstruction::Pop => vec![self.get_opcode()],
            ByteCodeInstruction::NewList => vec![self.get_opcode()],
            ByteCodeInstruction::ListLen => vec![self.get_opcode()],
            ByteCodeInstruction::ListGet => vec![self.get_opcode()],
            ByteCodeInstruction::PushBlock { index } => vec![self.get_opcode(), *index],
            ByteCodeInstruction::Load { index } => vec![self.get_opcode(), *index],
            ByteCodeInstruction::Store { index } => vec![self.get_opcode(), *index],
            ByteCodeInstruction::Dup => vec![self.get_opcode()],
            ByteCodeInstruction::Over => vec![self.get_opcode()],
            ByteCodeInstruction::Rot => vec![self.get_opcode()],
            ByteCodeInstruction::Swap => vec![self.get_opcode()],
            ByteCodeInstruction::Inc => vec![self.get_opcode()],
            ByteCodeInstruction::Dec => vec![self.get_opcode()],
            ByteCodeInstruction::Add => vec![self.get_opcode()],
            ByteCodeInstruction::Sub => vec![self.get_opcode()],
            ByteCodeInstruction::Mul => vec![self.get_opcode()],
            ByteCodeInstruction::Div => vec![self.get_opcode()],
            ByteCodeInstruction::Mod => vec![self.get_opcode()],
            ByteCodeInstruction::Gt => vec![self.get_opcode()],
            ByteCodeInstruction::Lt => vec![self.get_opcode()],
            ByteCodeInstruction::GtEq => vec![self.get_opcode()],
            ByteCodeInstruction::LtEq => vec![self.get_opcode()],
            ByteCodeInstruction::Eq => vec![self.get_opcode()],
            ByteCodeInstruction::Print => vec![self.get_opcode()],
            ByteCodeInstruction::PrintList => vec![self.get_opcode()],
            ByteCodeInstruction::Label(label) => vec![self.get_opcode(), *label],
            ByteCodeInstruction::CallStatic { index } => vec![self.get_opcode(), *index],
            ByteCodeInstruction::CallDynamic => vec![self.get_opcode()],
            ByteCodeInstruction::Jump { label } => vec![self.get_opcode(), *label],
            ByteCodeInstruction::JumpIfFalse { label } => vec![self.get_opcode(), *label],
            ByteCodeInstruction::Return => vec![self.get_opcode()],
            ByteCodeInstruction::PrintBool => vec![self.get_opcode()],
        }
    }

    pub fn decode(opcode: usize, arguments: &[usize]) -> (ByteCodeInstruction, usize) {
        match opcode {
            0x01 => (ByteCodeInstruction::Push(arguments[0]), 2), // opcode + 1 argument
            0x02 => (ByteCodeInstruction::Pop, 1),
            0x03 => (ByteCodeInstruction::NewList, 1),
            0x04 => (ByteCodeInstruction::ListLen, 1),
            0x05 => (ByteCodeInstruction::ListGet, 1),
            // 0x06 => (ByteCodeInstruction::ListSet, 1),
            0x07 => (
                ByteCodeInstruction::PushBlock {
                    index: arguments[0],
                },
                2,
            ),
            0x08 => (
                ByteCodeInstruction::Load {
                    index: arguments[0],
                },
                2,
            ),
            0x09 => (
                ByteCodeInstruction::Store {
                    index: arguments[0],
                },
                2,
            ),
            0x0A => (ByteCodeInstruction::Dup, 1),
            0x0B => (ByteCodeInstruction::Over, 1),
            0x0C => (ByteCodeInstruction::Rot, 1),
            0x0D => (ByteCodeInstruction::Swap, 1),
            0x0E => (ByteCodeInstruction::Add, 1),
            0x0F => (ByteCodeInstruction::Sub, 1),
            0x10 => (ByteCodeInstruction::Mul, 1),
            0x11 => (ByteCodeInstruction::Div, 1),
            0x12 => (ByteCodeInstruction::Mod, 1),
            0x13 => (ByteCodeInstruction::Gt, 1),
            0x14 => (ByteCodeInstruction::Lt, 1),
            0x15 => (ByteCodeInstruction::GtEq, 1),
            0x16 => (ByteCodeInstruction::LtEq, 1),
            0x17 => (ByteCodeInstruction::Eq, 1),
            0x18 => (ByteCodeInstruction::Print, 1),
            0x19 => (ByteCodeInstruction::PrintList, 1),
            0x1A => (ByteCodeInstruction::Label(arguments[0]), 2),
            0x1B => (
                ByteCodeInstruction::CallStatic {
                    index: arguments[0],
                },
                2,
            ),
            0x1C => (ByteCodeInstruction::CallDynamic, 1),
            0x1D => (
                ByteCodeInstruction::Jump {
                    label: arguments[0],
                },
                2,
            ),
            0x1E => (
                ByteCodeInstruction::JumpIfFalse {
                    label: arguments[0],
                },
                2,
            ),
            0x1F => (ByteCodeInstruction::Return, 1),
            0x20 => (ByteCodeInstruction::Inc, 1),
            0x21 => (ByteCodeInstruction::Dec, 1),
            0x22 => (ByteCodeInstruction::PrintBool, 1),
            _ => todo!("unhandled opcode {}", opcode),
        }
    }
}

pub struct Lowerer {
    next_label: usize,
    pub constant_pool: Vec<String>,
    functions: Vec<(Vec<TypeKind>, Vec<TypeKind>)>,
    bindings: HashMap<String, usize>,
    fns_to_emit: HashMap<String, Vec<ByteCodeInstruction>>,
    locals_count: usize,
    max_locals: usize,
}

pub struct StackFrame {
    pub instructions: Vec<ByteCodeInstruction>,
    pub max_locals: usize,
}

impl Lowerer {
    pub fn new() -> Lowerer {
        Lowerer {
            next_label: 0,
            constant_pool: Vec::new(),
            functions: Vec::new(),
            bindings: HashMap::new(),
            fns_to_emit: HashMap::new(),
            locals_count: 0,
            max_locals: 0,
        }
    }

    pub fn lower(&mut self, ops: &[TypedOp]) -> Vec<(String, StackFrame)> {
        let mut result: Vec<(String, StackFrame)> = Vec::new();

        let bytecode = self.lower_ops(ops);
        let frame = StackFrame {
            instructions: bytecode,
            max_locals: self.max_locals,
        };

        // println!("stack frame max locals: {}", frame.max_locals);

        self.locals_count = 0;
        self.bindings = HashMap::new();

        for (name, fn_to_emit) in &self.fns_to_emit {
            let frame = StackFrame {
                instructions: fn_to_emit.clone(),
                max_locals: self.max_locals,
            };
            self.locals_count = 0;
            self.bindings = HashMap::new();

            result.push((name.clone(), frame));
        }

        result.push(("main".to_string(), frame));
        result
    }

    fn lower_ops(&mut self, ops: &[TypedOp]) -> Vec<ByteCodeInstruction> {
        let mut bytecode = Vec::new();

        for op in ops {
            let bytecode_ops = self.lower_op(op);
            bytecode.extend(bytecode_ops);
        }

        bytecode
    }

    fn lower_op(&mut self, op: &TypedOp) -> Vec<ByteCodeInstruction> {
        match &op.kind {
            TypedOpKind::PushInt(value) => vec![ByteCodeInstruction::Push(*value as usize)],
            TypedOpKind::PushBool(value) => vec![ByteCodeInstruction::Push(*value as usize)],
            TypedOpKind::PushList(elements) => {
                let mut ops = Vec::new();
                for element in elements.iter().rev() {
                    ops.extend(self.lower_op(element));
                }
                ops.push(ByteCodeInstruction::Push(elements.len()));
                ops.push(ByteCodeInstruction::NewList);
                ops
            }
            TypedOpKind::PushBlock(ops) => {
                let index = self.constant_pool.len();

                let mut bytecode = self.lower_ops(ops);
                bytecode.push(ByteCodeInstruction::Return);

                self.fns_to_emit
                    .insert(format!("block_{}", index), bytecode);

                self.constant_pool.insert(index, format!("block_{}", index));

                vec![ByteCodeInstruction::PushBlock { index }]
            }
            TypedOpKind::Dup => {
                if let TypeKind::List(_) = &op.ins[0] {
                    return self.duplicate_list();
                }
                vec![ByteCodeInstruction::Dup]
            }
            TypedOpKind::Over => vec![ByteCodeInstruction::Over],
            TypedOpKind::Rot => vec![ByteCodeInstruction::Rot],
            TypedOpKind::Swap => vec![ByteCodeInstruction::Swap],
            TypedOpKind::Pop => vec![ByteCodeInstruction::Pop],
            TypedOpKind::Plus => vec![ByteCodeInstruction::Add],
            TypedOpKind::Minus => vec![ByteCodeInstruction::Sub],
            TypedOpKind::Multiply => vec![ByteCodeInstruction::Mul],
            TypedOpKind::Divide => vec![ByteCodeInstruction::Div],
            TypedOpKind::Modulo => vec![ByteCodeInstruction::Mod],
            TypedOpKind::GreaterThan => vec![ByteCodeInstruction::Gt],
            TypedOpKind::GreaterThanEquals => vec![ByteCodeInstruction::GtEq],
            TypedOpKind::LessThan => vec![ByteCodeInstruction::Lt],
            TypedOpKind::LessThanEquals => vec![ByteCodeInstruction::LtEq],
            TypedOpKind::Equals => vec![ByteCodeInstruction::Eq],
            TypedOpKind::Not => vec![ByteCodeInstruction::Push(0), ByteCodeInstruction::Eq],
            TypedOpKind::Map => {
                let func_idx = self.next_local();
                let list_idx = self.next_local();
                let index_idx = self.next_local();

                let cond = self.next_label();
                let end = self.next_label();

                //[list_ptr func_ptr]
                vec![
                    ByteCodeInstruction::Store { index: func_idx },
                    ByteCodeInstruction::Store { index: list_idx },
                    //init index with len
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::ListLen,
                    ByteCodeInstruction::Store { index: index_idx },
                    //init loop
                    //Prepare loop
                    ByteCodeInstruction::Label(cond),
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::Push(0),
                    //Is index > 0?
                    ByteCodeInstruction::Gt,
                    ByteCodeInstruction::JumpIfFalse { label: end },
                    //Decrement the index before performing the get
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::Dec,
                    ByteCodeInstruction::Store { index: index_idx },
                    //Get list[index]
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::ListGet,
                    //[el]
                    ByteCodeInstruction::Load { index: func_idx },
                    //[el func_ptr]
                    ByteCodeInstruction::CallDynamic,
                    //['el...]
                    ByteCodeInstruction::Jump { label: cond },
                    ByteCodeInstruction::Label(end),
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::ListLen,
                    ByteCodeInstruction::NewList,
                ]
            }
            TypedOpKind::Filter => {
                let func_idx = self.next_local();
                let list_idx = self.next_local();
                let index_idx = self.next_local();
                let count_idx = self.next_local();

                let cond = self.next_label();
                let end = self.next_label();

                //[list_ptr func_ptr]
                vec![
                    ByteCodeInstruction::Store { index: func_idx },
                    ByteCodeInstruction::Store { index: list_idx },
                    //init index with len
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::ListLen,
                    ByteCodeInstruction::Store { index: index_idx },
                    //init count with 0
                    ByteCodeInstruction::Push(0),
                    ByteCodeInstruction::Store { index: count_idx },
                    //init loop
                    //Prepare loop
                    ByteCodeInstruction::Label(cond),
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::Push(0),
                    //Is index > 0?
                    ByteCodeInstruction::Gt,
                    ByteCodeInstruction::JumpIfFalse { label: end },
                    //Decrement the index before performing the get
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::Dec,
                    ByteCodeInstruction::Store { index: index_idx },
                    //Get list[index]
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::ListGet,
                    //[el]
                    ByteCodeInstruction::Load { index: func_idx },
                    //[el func_ptr]
                    ByteCodeInstruction::CallDynamic,
                    //[true/false...]
                    //Jump back to cond if predicate failed
                    ByteCodeInstruction::JumpIfFalse { label: cond },
                    //else put the element onto the stack
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::ListGet,
                    //Increment element count
                    ByteCodeInstruction::Load { index: count_idx },
                    ByteCodeInstruction::Inc,
                    ByteCodeInstruction::Store { index: count_idx },
                    //loop
                    ByteCodeInstruction::Jump { label: cond },
                    ByteCodeInstruction::Label(end),
                    //new list from only the elements that passed the predicate
                    ByteCodeInstruction::Load { index: count_idx },
                    ByteCodeInstruction::NewList,
                ]
            }
            TypedOpKind::Fold => {
                let func_idx = self.next_local();
                let list_idx = self.next_local();
                let index_idx = self.next_local();
                let acc_idx = self.next_local();

                let cond = self.next_label();
                let end = self.next_label();

                //[list_ptr acc func_ptr]
                vec![
                    ByteCodeInstruction::Store { index: func_idx },
                    ByteCodeInstruction::Store { index: acc_idx },
                    ByteCodeInstruction::Store { index: list_idx },
                    //init index with len
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::ListLen,
                    ByteCodeInstruction::Store { index: index_idx },
                    //init loop
                    //Prepare loop
                    ByteCodeInstruction::Label(cond),
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::Push(0),
                    //Is index > 0?
                    ByteCodeInstruction::Gt,
                    ByteCodeInstruction::JumpIfFalse { label: end },
                    //Decrement the index before performing the get
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::Dec,
                    ByteCodeInstruction::Store { index: index_idx },
                    //Get list[index]
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::ListGet,
                    //Get accumulator
                    ByteCodeInstruction::Load { index: acc_idx },
                    //[el acc]
                    ByteCodeInstruction::Load { index: func_idx },
                    //[el acc func_ptr]
                    ByteCodeInstruction::CallDynamic,
                    //['el...]
                    ByteCodeInstruction::Store { index: acc_idx },
                    ByteCodeInstruction::Jump { label: cond },
                    ByteCodeInstruction::Label(end),
                    ByteCodeInstruction::Load { index: acc_idx },
                ]
            }
            TypedOpKind::Foreach => {
                let func_idx = self.next_local();
                let list_idx = self.next_local();
                let index_idx = self.next_local();

                let cond = self.next_label();
                let end = self.next_label();

                //[list_ptr func_ptr]
                vec![
                    ByteCodeInstruction::Store { index: func_idx },
                    ByteCodeInstruction::Store { index: list_idx },
                    //init index with 0
                    ByteCodeInstruction::Push(0),
                    ByteCodeInstruction::Store { index: index_idx },
                    //init loop
                    //Prepare loop
                    ByteCodeInstruction::Label(cond),
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::ListLen,
                    //Is index < len?
                    ByteCodeInstruction::Lt,
                    ByteCodeInstruction::JumpIfFalse { label: end },
                    //Get list[index]
                    ByteCodeInstruction::Load { index: list_idx },
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::ListGet,
                    //[el]
                    ByteCodeInstruction::Load { index: func_idx },
                    //[el func_ptr]
                    ByteCodeInstruction::CallDynamic,
                    //Increment the index
                    ByteCodeInstruction::Load { index: index_idx },
                    ByteCodeInstruction::Inc,
                    ByteCodeInstruction::Store { index: index_idx },
                    //Jump back to the condition
                    ByteCodeInstruction::Jump { label: cond },
                    ByteCodeInstruction::Label(end),
                ]
            }
            TypedOpKind::Print => match &op.ins[0] {
                TypeKind::List(_) => vec![ByteCodeInstruction::PrintList],
                TypeKind::Bool => vec![ByteCodeInstruction::PrintBool],
                _ => vec![ByteCodeInstruction::Print],
            },
            TypedOpKind::Len => vec![ByteCodeInstruction::ListLen],
            TypedOpKind::DefineFunction { name, block } => {
                if let TypedOpKind::PushBlock(ops) = &block.kind {
                    let mut bytecode = Vec::new();
                    for op in ops {
                        bytecode.extend(self.lower_op(op));
                    }
                    bytecode.push(ByteCodeInstruction::Return);
                    self.functions.push((block.ins.clone(), block.outs.clone()));
                    self.constant_pool.push(name.clone());

                    self.fns_to_emit.insert(name.clone(), bytecode);

                    vec![]
                } else {
                    unreachable!()
                }
            }
            TypedOpKind::Do => {
                vec![ByteCodeInstruction::CallDynamic]
            }
            TypedOpKind::Call(name) => {
                let index = self.constant_pool.iter().position(|n| n == name).unwrap();
                vec![ByteCodeInstruction::CallStatic { index }]
            }
            TypedOpKind::Binding { bindings, body } => {
                let mut bytecode = Vec::new();

                for binding in bindings {
                    let local = self.next_local();
                    bytecode.push(ByteCodeInstruction::Store { index: local });
                    self.bindings.insert(binding.clone(), local);
                }

                for op in body {
                    bytecode.extend(self.lower_op(op));
                }

                //Keep track of the maximum number of locals we use over the binding
                self.max_locals = max(self.max_locals, self.locals_count);

                //Unbind locals to reuse their space on the stack
                for binding in bindings {
                    self.locals_count -= 1;
                    self.bindings.remove(binding);
                }

                bytecode
            }
            TypedOpKind::Value(name) => {
                let binding = self.bindings.get(name).unwrap();

                vec![ByteCodeInstruction::Load { index: *binding }]
            }
            TypedOpKind::Identity => {
                vec![]
            }
            TypedOpKind::If { body, else_body } => {
                let end = self.next_label();

                let mut body_bytecode = Vec::new();
                for op in body {
                    body_bytecode.extend(self.lower_op(op));
                }

                match else_body {
                    Some(else_body) => {
                        let else_label = self.next_label();

                        let mut else_body_bytecode = Vec::new();
                        for op in else_body {
                            else_body_bytecode.extend(self.lower_op(op));
                        }
                        //[cond]
                        let mut bytecode =
                            vec![ByteCodeInstruction::JumpIfFalse { label: else_label }];
                        bytecode.extend(body_bytecode);
                        bytecode.push(ByteCodeInstruction::Jump { label: end });
                        bytecode.push(ByteCodeInstruction::Label(else_label));
                        bytecode.extend(else_body_bytecode);
                        bytecode.push(ByteCodeInstruction::Label(end));

                        bytecode
                    }
                    None => {
                        //[cond]
                        let mut bytecode = vec![ByteCodeInstruction::JumpIfFalse { label: end }];
                        bytecode.extend(body_bytecode);
                        bytecode.push(ByteCodeInstruction::Label(end));

                        bytecode
                    }
                }
            }
            _ => todo!("lowering {:?} is not yet implemented", op.kind),
        }
    }

    //Helper method for the code to duplicate a list on the stack
    fn duplicate_list(&mut self) -> Vec<ByteCodeInstruction> {
        //[list_ptr func_ptr]
        let cond = self.next_label();
        let end = self.next_label();

        let list_idx = self.next_local();
        let counter_idx = self.next_local();

        vec![
            //[list_ptr]
            ByteCodeInstruction::Store { index: list_idx },
            ByteCodeInstruction::Load { index: list_idx },
            //Create counter starting at len to count down
            ByteCodeInstruction::ListLen,
            ByteCodeInstruction::Store { index: counter_idx },
            //Prepare loop
            ByteCodeInstruction::Label(cond),
            ByteCodeInstruction::Push(0),
            ByteCodeInstruction::Load { index: counter_idx },
            //Is counter > 0?
            ByteCodeInstruction::Gt,
            ByteCodeInstruction::JumpIfFalse { label: end },
            //Decrement the counter before performing the get
            ByteCodeInstruction::Load { index: counter_idx },
            ByteCodeInstruction::Dec,
            ByteCodeInstruction::Store { index: counter_idx },
            //Push list[len - counter] onto the stack
            ByteCodeInstruction::Load { index: list_idx },
            ByteCodeInstruction::Load { index: counter_idx },
            ByteCodeInstruction::ListGet,
            ByteCodeInstruction::Jump { label: cond },
            ByteCodeInstruction::Label(end),
            //Create new list of the same size from the elements now on the stack
            ByteCodeInstruction::Load { index: list_idx },
            ByteCodeInstruction::ListLen,
            ByteCodeInstruction::NewList,
            //Restore the stack to be [orig new]
            ByteCodeInstruction::Load { index: list_idx },
            ByteCodeInstruction::Swap,
        ]
    }

    fn next_label(&mut self) -> usize {
        let label = self.next_label;
        self.next_label += 1;
        label
    }

    fn next_local(&mut self) -> usize {
        let local = self.locals_count;
        self.locals_count += 1;
        local
    }

    //TODO: this will enable String literals in future but we don't need it now
    // fn next_const(&mut self, name: String) -> usize {
    //     let index = self.constant_pool.len();
    //     self.constant_pool.push(name);
    //     index
    // }
}
