use crate::diagnostic::Diagnostic;
use crate::lexer::Span;
use crate::parser::{Op, OpKind};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
enum TypeKind {
    Int,
    List(Box<TypeKind>),
    Generic(usize),
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Int => write!(f, "int"),
            TypeKind::List(el_type) => write!(f, "[{}]", el_type),
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
                    Some((type_kind, _span)) => match input {
                        //TODO fix the clone
                        TypeKind::Generic(index) => match self.erasures.clone().get(index) {
                            Some(erased) => self.expect_type(op, &type_kind, erased),
                            None => self.erasures.insert(index, type_kind),
                        },
                        _ => self.expect_type(op, &type_kind, &input),
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

    fn expect_type(&mut self, op: &Op, actual: &TypeKind, expected: &TypeKind) {
        if expected != actual {
            self.diagnostics.push(Diagnostic::report_error(
                format!("Expected {} but got {}", expected, actual),
                op.span,
            ))
        }
    }

    fn create_generic(&mut self) -> usize {
        let generic_index = self.next_generic_index;
        self.next_generic_index += 1;
        generic_index
    }

    fn get_signature(&mut self, op_kind: &OpKind) -> (Vec<TypeKind>, Vec<TypeKind>) {
        match op_kind {
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
                        Some(type_kind) => self.expect_type(op, type_kind, out),
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
            OpKind::Plus | OpKind::Minus | OpKind::Multiply | OpKind::Divide | OpKind::Modulo => {
                (vec![TypeKind::Int, TypeKind::Int], vec![TypeKind::Int])
            }

            OpKind::Dup => {
                let index = self.create_generic();
                (
                    vec![TypeKind::Generic(index)],
                    vec![TypeKind::Generic(index), TypeKind::Generic(index)],
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
        }
    }
}
