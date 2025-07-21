use crate::typechecker::{TypeKind, TypedOp, TypedOpKind};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ByteCodeInstruction {
    Push(usize),
    Pop,
    NewList {
        length: usize,
    },
    ListLen,
    ListGet,
    ListSet,
    PushBlock {
        index: usize,
    },
    Load {
        index: usize,
    },
    Store {
        index: usize,
    },
    Dup,
    Over,
    Rot,
    Swap,
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
    PrintList,
    Label(usize),
    Call {
        index: usize,
        in_count: usize,
        out_count: usize,
    },
    Jump {
        label: usize,
    },
    JumpIfFalse {
        label: usize,
    },
    Return,
    //TODO: I don't think these should be bytecode intrinsics
    Map,
}

impl ByteCodeInstruction {
    fn get_opcode(&self) -> usize {
        match self {
            ByteCodeInstruction::Push(_) => 0x01,
            ByteCodeInstruction::Pop => 0x02,
            ByteCodeInstruction::NewList { .. } => 0x03,
            ByteCodeInstruction::ListLen => 0x04,
            ByteCodeInstruction::ListGet => 0x05,
            ByteCodeInstruction::ListSet => 0x06,
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
            ByteCodeInstruction::Call { .. } => 0x1B,
            ByteCodeInstruction::Jump { .. } => 0x1C,
            ByteCodeInstruction::JumpIfFalse { .. } => 0x1D,
            ByteCodeInstruction::Return => 0x1E,
            ByteCodeInstruction::Map => 0x1F,
        }
    }

    pub fn to_binary(self) -> Vec<usize> {
        match self {
            ByteCodeInstruction::Push(value) => vec![self.get_opcode(), value],
            ByteCodeInstruction::Pop => vec![self.get_opcode()],
            ByteCodeInstruction::NewList { length } => vec![self.get_opcode(), length],
            ByteCodeInstruction::ListLen => vec![self.get_opcode()],
            ByteCodeInstruction::ListGet => vec![self.get_opcode()],
            ByteCodeInstruction::ListSet => vec![self.get_opcode()],
            ByteCodeInstruction::PushBlock { index } => vec![self.get_opcode(), index],
            ByteCodeInstruction::Load { index } => vec![self.get_opcode(), index],
            ByteCodeInstruction::Store { index } => vec![self.get_opcode(), index],
            ByteCodeInstruction::Dup => vec![self.get_opcode()],
            ByteCodeInstruction::Over => vec![self.get_opcode()],
            ByteCodeInstruction::Rot => vec![self.get_opcode()],
            ByteCodeInstruction::Swap => vec![self.get_opcode()],
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
            ByteCodeInstruction::Label(label) => vec![self.get_opcode(), label],
            ByteCodeInstruction::Call {
                index,
                in_count,
                out_count,
            } => vec![self.get_opcode(), index, in_count, out_count],
            ByteCodeInstruction::Jump { label } => vec![self.get_opcode(), label],
            ByteCodeInstruction::JumpIfFalse { label } => vec![self.get_opcode(), label],
            ByteCodeInstruction::Return => vec![self.get_opcode()],
            ByteCodeInstruction::Map => vec![self.get_opcode()],
        }
    }

    pub fn decode(opcode: usize, arguments: &[usize]) -> (ByteCodeInstruction, usize) {
        match opcode {
            0x01 => (ByteCodeInstruction::Push(arguments[0]), 2), // opcode + 1 argument
            0x02 => (ByteCodeInstruction::Pop, 1),
            0x03 => (
                ByteCodeInstruction::NewList {
                    length: arguments[0],
                },
                2,
            ),
            0x04 => (ByteCodeInstruction::ListLen, 1),
            0x05 => (ByteCodeInstruction::ListGet, 1),
            0x06 => (ByteCodeInstruction::ListSet, 1),
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
                ByteCodeInstruction::Call {
                    index: arguments[0],
                    in_count: arguments[1],
                    out_count: arguments[2],
                },
                4,
            ),
            0x1C => (
                ByteCodeInstruction::Jump {
                    label: arguments[0],
                },
                2,
            ),
            0x1D => (
                ByteCodeInstruction::JumpIfFalse {
                    label: arguments[0],
                },
                2,
            ),
            0x1E => (ByteCodeInstruction::Return, 1),
            0x1F => (ByteCodeInstruction::Map, 1),
            _ => todo!("unhandled opcode {}", opcode),
        }
    }
}

pub struct Lowerer {
    next_label: usize,
    pub constant_pool: Vec<String>,
    functions: Vec<(Vec<TypeKind>, Vec<TypeKind>)>,
    fns_to_emit: HashMap<String, Vec<ByteCodeInstruction>>,
}

impl Lowerer {
    pub fn new() -> Lowerer {
        Lowerer {
            next_label: 0,
            constant_pool: Vec::new(),
            functions: Vec::new(),
            fns_to_emit: HashMap::new(),
        }
    }

    pub fn lower(&mut self, ops: &[TypedOp]) -> Vec<(String, Vec<ByteCodeInstruction>)> {
        let mut result: Vec<(String, Vec<ByteCodeInstruction>)> = Vec::new();

        let bytecode = self.lower_ops(ops);

        for (name, fn_to_emit) in &self.fns_to_emit {
            result.push((name.clone(), fn_to_emit.clone()));
        }

        result.push(("start".to_string(), bytecode));
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
                    //TODO: this is not right, need TypedOp to have a TypedOpKind so list element type info is not lost
                    //      also needs to be recursive, if it were a List<TypedOpKind> could just do ops.extend(self.lower_op(element))
                    ops.extend(self.lower_op(element));
                }
                ops.push(ByteCodeInstruction::NewList {
                    length: elements.len(),
                });
                ops
            }
            TypedOpKind::PushBlock(ops) => {
                let label = self.next_label();

                let mut bytecode = self.lower_ops(ops);
                bytecode.push(ByteCodeInstruction::Return);

                self.fns_to_emit
                    .insert(format!("block_{}", label), bytecode);

                vec![ByteCodeInstruction::PushBlock { index: label }]
            }
            TypedOpKind::Dup => vec![ByteCodeInstruction::Dup],
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
                vec![ByteCodeInstruction::Map]
            }
            TypedOpKind::Print => match &op.ins[0] {
                TypeKind::List(_) => vec![ByteCodeInstruction::PrintList],
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
            TypedOpKind::Call(name) => {
                let index = self.constant_pool.iter().position(|n| n == name).unwrap();
                let (ins, outs) = self.functions.get(index).unwrap();

                vec![ByteCodeInstruction::Call {
                    index,
                    in_count: ins.len(),
                    out_count: outs.len(),
                }]
            }
            _ => todo!("lowering {:?} is not yet implemented", op.kind),
        }
    }

    fn next_label(&mut self) -> usize {
        let label = self.next_label;
        self.next_label += 1;
        label
    }
}
