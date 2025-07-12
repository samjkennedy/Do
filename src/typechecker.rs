use crate::diagnostic::Diagnostic;
use crate::lexer::Span;
use crate::parser::{Op, OpKind};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
enum TypeKind {
    Bool,
    Int,
    List(Box<TypeKind>),
    Block {
        ins: Vec<TypeKind>,
        outs: Vec<TypeKind>,
    },
    Generic(usize),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Int => write!(f, "int"),
            TypeKind::List(el_type) => write!(f, "[{}]", el_type),
            TypeKind::Block { ins, outs } => write!(
                f,
                "fn [{} -- {}]",
                ins.iter()
                    .map(|el| el.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
                outs.iter()
                    .map(|el| el.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            TypeKind::Generic(n) => write!(f, "<{}>", n),
        }
    }
}

pub struct TypeChecker {
    type_stack: Vec<(TypeKind, Span)>,
    pub diagnostics: Vec<Diagnostic>,
    erasures: Vec<TypeKind>,
    next_generic_index: usize,
}

impl TypeChecker {
    pub fn new() -> TypeChecker {
        TypeChecker {
            type_stack: Vec::new(),
            diagnostics: Vec::new(),
            erasures: Vec::new(),
            next_generic_index: 0,
        }
    }

    pub fn type_check(&mut self, ops: &Vec<Op>) {
        for op in ops {
            let (ins, outs) = self.get_signature(&op.kind);

            for input in ins {
                match self.type_stack.pop() {
                    Some((type_kind, span)) => match input {
                        //TODO fix the clone
                        TypeKind::Generic(index) => match self.erasures.clone().get(index) {
                            //TODO: is it clearer to report the op span or the type's span?
                            //      should report both really
                            Some(erased) => self.expect_type(&type_kind, erased, op.span),
                            None => self.erasures.insert(index, type_kind),
                        },
                        _ => self.expect_type(&type_kind, &input, span),
                    },
                    None => self.diagnostics.push(Diagnostic::report_error(
                        format!("Expected {} but stack was empty", input),
                        op.span,
                    )),
                }
            }

            for output in outs {
                match output {
                    TypeKind::Generic(index) => match self.erasures.get(index) {
                        Some(erased) => self.type_stack.push((erased.clone(), op.span)),
                        None => self.type_stack.push((output, op.span)),
                    },
                    _ => self.type_stack.push((output, op.span)),
                }
            }
        }

        for (type_kind, span) in &self.type_stack {
            self.diagnostics.push(Diagnostic::report_error(
                format!(
                    "Type stack must be empty at the end of the program, but got {}",
                    type_kind
                ),
                *span,
            ))
        }
    }

    fn expect_type(&mut self, actual: &TypeKind, expected: &TypeKind, span: Span) {
        match (actual, expected) {
            (
                TypeKind::Block {
                    ins: actual_ins,
                    outs: actual_outs,
                },
                TypeKind::Block {
                    ins: expected_ins,
                    outs: expected_outs,
                },
            ) => {
                if actual_ins.len() != expected_ins.len()
                    || actual_outs.len() != expected_outs.len()
                {
                    self.diagnostics.push(Diagnostic::report_error(
                        format!("Expected {} but got {}", expected, actual),
                        span,
                    ))
                }
                for (actual_in, expected_in) in actual_ins.iter().zip(expected_ins.iter()) {
                    self.expect_type(actual_in, expected_in, span);
                }
                for (actual_out, expected_out) in actual_outs.iter().zip(expected_outs.iter()) {
                    self.expect_type(actual_out, expected_out, span);
                }
            }
            (TypeKind::Generic(index), expected) => todo!(),
            (actual, TypeKind::Generic(index)) => match self.erasures.clone().get(*index) {
                None => self.erasures.insert(*index, actual.clone()),
                Some(type_kind) => self.expect_type(actual, type_kind, span),
            },
            _ => {
                if expected != actual {
                    self.diagnostics.push(Diagnostic::report_error(
                        format!("Expected {} but got {}", expected, actual),
                        span,
                    ))
                }
            }
        }
    }

    fn create_generic(&mut self) -> usize {
        let generic_index = self.next_generic_index;
        self.next_generic_index += 1;

        generic_index
    }

    fn get_signature(&mut self, op_kind: &OpKind) -> (Vec<TypeKind>, Vec<TypeKind>) {
        match op_kind {
            OpKind::PushBool(_) => (vec![], vec![TypeKind::Bool]),
            OpKind::PushInt(_) => (vec![], vec![TypeKind::Int]),
            OpKind::PushList(ops) => {
                let mut element_type: Option<TypeKind> = None;
                for op in ops {
                    let (ins, outs) = self.get_signature(&op.kind);
                    if !ins.is_empty() {
                        unreachable!()
                    }
                    if outs.len() != 1 {
                        unreachable!()
                    }
                    let out = outs.first().unwrap();
                    match &element_type {
                        Some(type_kind) => self.expect_type(type_kind, out, op.span),
                        None => element_type = Some(out.clone()),
                    }
                }
                match element_type {
                    None => {
                        let index = self.create_generic();
                        (
                            vec![],
                            vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                        )
                    }
                    Some(type_kind) => (vec![], vec![TypeKind::List(Box::new(type_kind))]),
                }
            }
            OpKind::PushBlock(ops) => {
                let mut ins: Vec<TypeKind> = Vec::new();
                let mut outs: Vec<TypeKind> = Vec::new();

                for op in ops {
                    let (op_ins, op_outs) = self.get_signature(&op.kind);

                    for op_in in op_ins {
                        match outs.pop() {
                            Some(TypeKind::Generic(index)) => {
                                if let Some(erased) = self.erasures.clone().get(index) {
                                    self.expect_type(erased, &op_in, op.span)
                                } else if self.erasures.clone().get(index).is_none() {
                                    self.erasures.insert(index, op_in)
                                }
                            }
                            Some(out) => self.expect_type(&out, &op_in, op.span),
                            None => match ins.pop() {
                                Some(type_kind) => self.expect_type(&type_kind, &op_in, op.span),
                                None => ins.push(op_in),
                            },
                        }
                    }

                    for op_out in op_outs {
                        match op_out {
                            TypeKind::Generic(index) => match self.erasures.get(index) {
                                Some(erased) => self.type_stack.push((erased.clone(), op.span)),
                                None => outs.push(op_out),
                            },
                            _ => outs.push(op_out),
                        }
                    }
                }

                let mut erased_ins = Vec::new();
                for block_in in ins {
                    match block_in {
                        TypeKind::Generic(index) => match self.erasures.clone().get(index) {
                            Some(type_kind) => erased_ins.push(type_kind.clone()),
                            None => erased_ins.push(block_in),
                        },
                        _ => erased_ins.push(block_in),
                    }
                }

                let mut erased_outs = Vec::new();
                for block_out in outs {
                    match block_out {
                        TypeKind::Generic(index) => match self.erasures.clone().get(index) {
                            Some(type_kind) => erased_outs.push(type_kind.clone()),
                            None => erased_outs.push(block_out),
                        },
                        _ => erased_outs.push(block_out),
                    }
                }

                (
                    vec![],
                    vec![TypeKind::Block {
                        ins: erased_ins,
                        outs: erased_outs,
                    }],
                )
            }
            OpKind::Plus | OpKind::Minus | OpKind::Multiply | OpKind::Divide | OpKind::Modulo => {
                (vec![TypeKind::Int, TypeKind::Int], vec![TypeKind::Int])
            }
            OpKind::LessThan
            | OpKind::GreaterThan
            | OpKind::LessThanEquals
            | OpKind::GreaterThanEquals => {
                (vec![TypeKind::Int, TypeKind::Int], vec![TypeKind::Bool])
            }
            OpKind::Equals => {
                let index = self.create_generic();
                (
                    vec![TypeKind::Generic(index), TypeKind::Generic(index)],
                    vec![TypeKind::Bool],
                )
            }
            OpKind::Not => (vec![TypeKind::Bool], vec![TypeKind::Bool]),
            OpKind::Dup => {
                let index = self.create_generic();
                (
                    vec![TypeKind::Generic(index)],
                    vec![TypeKind::Generic(index), TypeKind::Generic(index)],
                )
            }
            OpKind::Over => {
                let a = self.create_generic();
                let b = self.create_generic();

                (
                    vec![TypeKind::Generic(a), TypeKind::Generic(b)],
                    vec![
                        TypeKind::Generic(a),
                        TypeKind::Generic(b),
                        TypeKind::Generic(a),
                    ],
                )
            }
            OpKind::Pop => {
                let index = self.create_generic();

                (vec![TypeKind::Generic(index)], vec![])
            }
            OpKind::Rot => {
                let a = self.create_generic();
                let b = self.create_generic();
                let c = self.create_generic();
                (
                    vec![
                        TypeKind::Generic(a),
                        TypeKind::Generic(b),
                        TypeKind::Generic(c),
                    ],
                    vec![
                        TypeKind::Generic(b),
                        TypeKind::Generic(c),
                        TypeKind::Generic(a),
                    ],
                )
            }
            OpKind::Swap => {
                let a = self.create_generic();
                let b = self.create_generic();
                (
                    vec![TypeKind::Generic(a), TypeKind::Generic(b)],
                    vec![TypeKind::Generic(a), TypeKind::Generic(b)],
                )
            }
            OpKind::Print => {
                let index = self.create_generic();

                (vec![TypeKind::Generic(index)], vec![])
            }
            OpKind::Map => {
                let a = self.create_generic();
                let b = self.create_generic();

                (
                    vec![TypeKind::Block {
                        ins: vec![TypeKind::Generic(a)],
                        outs: vec![TypeKind::Generic(b)],
                    }],
                    vec![],
                )
            }
        }
    }
}
