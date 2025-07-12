use crate::diagnostic::Diagnostic;
use crate::parser::{Op, OpKind};
use std::cmp::PartialEq;

#[derive(Debug, Clone, Copy, PartialEq)]
enum TypeKind {
    Bool,
    Int,
    Generic(usize),
}

pub struct TypeChecker {
    type_stack: Vec<TypeKind>,
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
                    Some(type_kind) => match input {
                        //TODO fix the clone
                        TypeKind::Generic(index) => match self.erasures.clone().get(index) {
                            Some(erased) => self.expect_type(op, &type_kind, erased),
                            None => self.erasures.insert(index, type_kind),
                        },
                        _ => self.expect_type(op, &input, &type_kind),
                    },
                    None => self.diagnostics.push(Diagnostic::report_error(
                        format!("Expected {:?} but stack was empty", input),
                        op.span,
                    )),
                }
            }

            for output in outs {
                match output {
                    TypeKind::Generic(index) => {
                        match self.erasures.get(index) {
                            Some(erased) => self.type_stack.push(*erased),
                            None => self.type_stack.push(output)
                        }
                    }
                    _ => self.type_stack.push(output)
                }
            }
        }
    }

    fn expect_type(&mut self, op: &Op, actual: &TypeKind, expected: &TypeKind) {
        if expected != actual {
            self.diagnostics.push(Diagnostic::report_error(
                format!("Expected {:?} but got {:?}", expected, actual),
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
