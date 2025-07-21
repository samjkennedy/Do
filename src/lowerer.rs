use crate::typechecker::{TypeKind, TypedOp, TypedOpKind};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
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

pub struct Lowerer {
    next_label: usize,
    pub constant_pool: Vec<String>,
    functions: Vec<(Vec<TypeKind>, Vec<TypeKind>)>,
    fns_to_emit: HashMap<String, Vec<OpCode>>,
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
    pub fn lower(&mut self, ops: &[TypedOp]) -> Vec<(String, Vec<OpCode>)> {
        let mut result: Vec<(String, Vec<OpCode>)> = Vec::new();

        let bytecode = self.lower_ops(ops);

        for (name, fn_to_emit) in &self.fns_to_emit {
            result.push((name.clone(), fn_to_emit.clone()));
        }

        result.push(("start".to_string(), bytecode));
        result
    }

    fn lower_ops(&mut self, ops: &[TypedOp]) -> Vec<OpCode> {
        let mut bytecode = Vec::new();

        for op in ops {
            let bytecode_ops = self.lower_op(op);
            bytecode.extend(bytecode_ops);
        }

        bytecode
    }

    fn lower_op(&mut self, op: &TypedOp) -> Vec<OpCode> {
        match &op.kind {
            TypedOpKind::PushInt(value) => vec![OpCode::Push(*value as usize)],
            TypedOpKind::PushBool(value) => vec![OpCode::Push(*value as usize)],
            TypedOpKind::PushList(elements) => {
                let mut ops = Vec::new();
                for element in elements.iter().rev() {
                    //TODO: this is not right, need TypedOp to have a TypedOpKind so list element type info is not lost
                    //      also needs to be recursive, if it were a List<TypedOpKind> could just do ops.extend(self.lower_op(element))
                    ops.extend(self.lower_op(element));
                }
                ops.push(OpCode::NewList {
                    length: elements.len(),
                });
                ops
            }
            TypedOpKind::PushBlock(ops) => {
                let label = self.next_label();

                let mut bytecode = self.lower_ops(ops);
                bytecode.push(OpCode::Return);

                self.fns_to_emit
                    .insert(format!("block_{}", label), bytecode);

                vec![OpCode::PushBlock { index: label }]
            }
            TypedOpKind::Dup => vec![OpCode::Dup],
            TypedOpKind::Over => vec![OpCode::Over],
            TypedOpKind::Rot => vec![OpCode::Rot],
            TypedOpKind::Swap => vec![OpCode::Swap],
            TypedOpKind::Pop => vec![OpCode::Pop],
            TypedOpKind::Plus => vec![OpCode::Add],
            TypedOpKind::Minus => vec![OpCode::Sub],
            TypedOpKind::Multiply => vec![OpCode::Mul],
            TypedOpKind::Divide => vec![OpCode::Div],
            TypedOpKind::Modulo => vec![OpCode::Mod],
            TypedOpKind::GreaterThan => vec![OpCode::Gt],
            TypedOpKind::GreaterThanEquals => vec![OpCode::GtEq],
            TypedOpKind::LessThan => vec![OpCode::Lt],
            TypedOpKind::LessThanEquals => vec![OpCode::LtEq],
            TypedOpKind::Equals => vec![OpCode::Eq],
            TypedOpKind::Not => vec![OpCode::Push(0), OpCode::Eq],
            TypedOpKind::Map => {
                vec![OpCode::Map]
            }
            TypedOpKind::Print => match &op.ins[0] {
                TypeKind::List(_) => vec![OpCode::PrintList],
                _ => vec![OpCode::Print],
            },
            TypedOpKind::Len => vec![OpCode::ListLen],
            TypedOpKind::DefineFunction { name, block } => {
                if let TypedOpKind::PushBlock(ops) = &block.kind {
                    let mut bytecode = Vec::new();
                    for op in ops {
                        bytecode.extend(self.lower_op(op));
                    }
                    bytecode.push(OpCode::Return);
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

                vec![OpCode::Call {
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
