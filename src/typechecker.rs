use crate::diagnostic::Diagnostic;
use crate::lexer::{Span, TokenKind};
use crate::parser::{Op, OpKind};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
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
            TypeKind::Generic(index) => write!(f, "<{}>", index),
        }
    }
}

#[derive(Clone)]
pub struct TypeChecker {
    fail_on_non_empty_stack: bool,
    pub type_stack: Vec<(TypeKind, Span)>,
    pub diagnostics: Vec<Diagnostic>,
    erasures: Vec<Option<TypeKind>>,
    next_generic_index: usize,
    functions: HashMap<String, (Vec<TypeKind>, Vec<TypeKind>)>,
}

impl TypeChecker {
    pub fn new(fail_on_non_empty_stack: bool) -> TypeChecker {
        TypeChecker {
            fail_on_non_empty_stack,
            type_stack: Vec::new(),
            diagnostics: Vec::new(),
            erasures: Vec::new(),
            next_generic_index: 0,
            functions: HashMap::new(),
        }
    }

    pub fn type_check(&mut self, ops: &Vec<Op>) {
        for op in ops {
            // println!("op: {:?}", op.kind);
            let (ins, outs) = self.get_signature(&op.kind, op.span);

            // println!("  ins: {:?}, outs: {:?}", ins, outs);

            for input in ins {
                match self.type_stack.pop() {
                    Some((type_kind, span)) => self.expect_type(&type_kind, &input, op.span),
                    None => self.diagnostics.push(Diagnostic::report_error(
                        format!("Expected {} but stack was empty", input),
                        op.span,
                    )),
                }
            }

            for output in outs {
                self.type_stack.push((output, op.span));
            }
        }

        if self.fail_on_non_empty_stack {
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
    }

    fn erase(&self, type_kind: &TypeKind) -> Option<TypeKind> {
        match type_kind {
            TypeKind::Generic(index) => match self.erasures.get(*index).unwrap() {
                Some(erasure) => self.erase(erasure),
                None => None,
            },
            TypeKind::List(element_type) => self
                .erase(element_type)
                .map(|erased_element_type| TypeKind::List(Box::new(erased_element_type))),
            TypeKind::Block { ins, outs } => {
                let erased_ins = ins
                    .iter()
                    .map(|i| self.erase(i).unwrap_or(i.clone()))
                    .collect();
                let erased_outs = outs
                    .iter()
                    .map(|i| self.erase(i).unwrap_or(i.clone()))
                    .collect();
                Some(TypeKind::Block {
                    ins: erased_ins,
                    outs: erased_outs,
                })
            }
            _ => Some(type_kind.clone()),
        }
    }

    fn expect_type(&mut self, actual: &TypeKind, expected: &TypeKind, span: Span) {
        self.expect_type_inner(actual, expected, actual, expected, span);
    }

    fn expect_type_inner(
        &mut self,
        actual: &TypeKind,
        expected: &TypeKind,
        original_actual: &TypeKind,
        original_expected: &TypeKind,
        span: Span,
    ) {
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
                        format!(
                            "Expected {} but got {}",
                            self.erase(original_expected).unwrap(),
                            self.erase(original_actual).unwrap()
                        ),
                        span,
                    ));
                }
                for (actual_in, expected_in) in actual_ins.iter().zip(expected_ins.iter()) {
                    self.expect_type_inner(
                        actual_in,
                        expected_in,
                        original_actual,
                        original_expected,
                        span,
                    );
                }
                for (actual_out, expected_out) in actual_outs.iter().zip(expected_outs.iter()) {
                    self.expect_type_inner(
                        actual_out,
                        expected_out,
                        original_actual,
                        original_expected,
                        span,
                    );
                }
            }
            //These generic/list pairings feel like they're patching over a mistake somewhere else...
            (TypeKind::Generic(index), TypeKind::List(rhs)) => {
                match self.erasures.get(*index).unwrap().clone() {
                    None => self.erase_generic(index, expected),
                    Some(type_kind) => self.expect_type_inner(
                        &type_kind,
                        rhs,
                        original_actual,
                        original_expected,
                        span,
                    ),
                }
            }
            //These generic/list pairings feel like they're patching over a mistake somewhere else...
            (TypeKind::List(lhs), TypeKind::Generic(index)) => {
                match self.erasures.get(*index).unwrap().clone() {
                    None => self.erase_generic(index, expected),
                    Some(type_kind) => self.expect_type_inner(
                        lhs,
                        &type_kind,
                        original_actual,
                        original_expected,
                        span,
                    ),
                }
            }
            (TypeKind::Generic(index), expected) => {
                match self.erasures.get(*index).unwrap().clone() {
                    None => self.erase_generic(index, expected),
                    Some(type_kind) => self.expect_type_inner(
                        &type_kind,
                        expected,
                        original_actual,
                        original_expected,
                        span,
                    ),
                }
            }
            (actual, TypeKind::Generic(index)) => {
                match self.erasures.get(*index).unwrap().clone() {
                    None => self.erase_generic(index, actual),
                    Some(type_kind) => self.expect_type_inner(
                        actual,
                        &type_kind,
                        original_actual,
                        original_expected,
                        span,
                    ),
                }
            }
            (TypeKind::List(lhs), TypeKind::List(rhs)) => {
                self.expect_type_inner(lhs, rhs, original_actual, original_expected, span);
            }
            _ => {
                if self.erase(expected) != self.erase(actual) {
                    self.diagnostics.push(Diagnostic::report_error(
                        format!(
                            "Expected {} but got {}",
                            self.erase(original_expected).unwrap(),
                            self.erase(original_actual).unwrap()
                        ),
                        span,
                    ));
                }
            }
        }
    }

    fn create_generic(&mut self) -> usize {
        let generic_index = self.next_generic_index;
        self.next_generic_index += 1;

        //None means an unknown generic for this index
        self.erasures.push(None);

        assert_eq!(self.erasures.len(), self.next_generic_index);

        generic_index
    }

    fn erase_generic(&mut self, index: &usize, erasure: &TypeKind) {
        let erased = self.erase(erasure);
        // println!("generic: {:?} erased to {:?}", erasure, erased);
        self.erasures[*index] = erased;
    }

    fn get_signature(&mut self, op_kind: &OpKind, span: Span) -> (Vec<TypeKind>, Vec<TypeKind>) {
        match op_kind {
            OpKind::PushBool(_) => (vec![], vec![TypeKind::Bool]),
            OpKind::PushInt(_) => (vec![], vec![TypeKind::Int]),
            OpKind::PushList(ops) => {
                let mut element_type: Option<TypeKind> = None;
                for op in ops {
                    let (ins, outs) = self.get_signature(&op.kind, op.span);
                    if !ins.is_empty() || outs.len() != 1 {
                        self.diagnostics.push(Diagnostic::report_error(
                            format!(
                                "List elements must have the signature [ -- <a>], got [{} -- {}]",
                                ins.iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join(" "),
                                outs.iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join(" ")
                            ),
                            op.span,
                        ));
                        //Return what we know just to keep parsing
                        return match element_type {
                            None => {
                                let index = self.create_generic();
                                (
                                    vec![],
                                    vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                                )
                            }
                            Some(type_kind) => (vec![], vec![TypeKind::List(Box::new(type_kind))]),
                        };
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
                let (ins, outs) = self.get_block_signature(ops);
                (vec![], vec![TypeKind::Block { ins, outs }])
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
            OpKind::Identity => {
                let index = self.create_generic();
                (
                    vec![TypeKind::Generic(index)],
                    vec![TypeKind::Generic(index)],
                )
            }
            OpKind::And => (vec![TypeKind::Bool, TypeKind::Bool], vec![TypeKind::Bool]),
            OpKind::Or => (vec![TypeKind::Bool, TypeKind::Bool], vec![TypeKind::Bool]),
            OpKind::Dup => {
                let index = self.create_generic();
                (
                    vec![TypeKind::Generic(index)],
                    vec![TypeKind::Generic(index), TypeKind::Generic(index)],
                )
            }
            OpKind::Len => {
                let index = self.create_generic();
                (vec![TypeKind::Generic(index)], vec![TypeKind::Int])
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
            OpKind::Concat => {
                let index = self.create_generic();
                (
                    vec![
                        TypeKind::List(Box::new(TypeKind::Generic(index))),
                        TypeKind::List(Box::new(TypeKind::Generic(index))),
                    ],
                    vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                )
            }
            OpKind::Do => {
                (
                    vec![TypeKind::Block {
                        ins: vec![], //TODO: Do should accept varargs
                        outs: vec![],
                    }],
                    vec![],
                )
            }
            OpKind::Filter => {
                let a = self.create_generic();

                (
                    vec![
                        TypeKind::Block {
                            ins: vec![TypeKind::Generic(a)],
                            outs: vec![TypeKind::Bool],
                        },
                        TypeKind::List(Box::new(TypeKind::Generic(a))),
                    ],
                    vec![TypeKind::List(Box::new(TypeKind::Generic(a)))],
                )
            }
            OpKind::Fold => {
                let a = self.create_generic();
                (
                    vec![
                        TypeKind::Generic(a),
                        TypeKind::Block {
                            ins: vec![TypeKind::Generic(a), TypeKind::Generic(a)],
                            outs: vec![TypeKind::Generic(a)],
                        },
                        TypeKind::List(Box::new(TypeKind::Generic(a))),
                    ],
                    vec![TypeKind::Generic(a)],
                )
            }
            OpKind::Foreach => {
                let a = self.create_generic();
                (
                    vec![
                        TypeKind::Block {
                            ins: vec![TypeKind::Generic(a)],
                            outs: vec![],
                        },
                        TypeKind::List(Box::new(TypeKind::Generic(a))),
                    ],
                    vec![],
                )
            }
            OpKind::Map => {
                let a = self.create_generic();
                let b = self.create_generic();

                (
                    vec![
                        TypeKind::Block {
                            ins: vec![TypeKind::Generic(a)],
                            outs: vec![TypeKind::Generic(b)],
                        },
                        TypeKind::List(Box::new(TypeKind::Generic(a))),
                    ],
                    vec![TypeKind::List(Box::new(TypeKind::Generic(b)))],
                )
            }
            OpKind::DumpStack => {
                for (type_kind, span) in &self.type_stack {
                    println!(
                        "{} defined at {:?}",
                        self.erase(type_kind).unwrap_or(type_kind.clone()),
                        span
                    );
                }
                (vec![], vec![])
            }
            OpKind::DefineFunction { identifier, body } => {
                if let TokenKind::Identifier(name) = &identifier.kind {
                    if let OpKind::PushBlock(ops) = &body.kind {
                        let (ins, outs) = self.get_block_signature(ops);

                        self.functions.insert(name.clone(), (ins, outs));

                        (vec![], vec![]) //declaring a function doesn't affect the stack
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            OpKind::Call(name) => {
                match self.functions.get(name) {
                    Some((ins, outs)) => (ins.clone(), outs.clone()),
                    None => {
                        self.diagnostics.push(Diagnostic::report_error(
                            format!("no such function `{}`", name),
                            span,
                        ));
                        (vec![], vec![]) //return nothing to keep going
                    }
                }
            }
        }
    }

    fn get_block_signature(&mut self, ops: &Vec<Op>) -> (Vec<TypeKind>, Vec<TypeKind>) {
        let mut ins: Vec<TypeKind> = Vec::new();
        let mut outs: Vec<TypeKind> = Vec::new();

        for op in ops {
            let (op_ins, op_outs) = self.get_signature(&op.kind, op.span);

            // println!("  op: {:?}, op_ins: {:?}, op_outs: {:?}", op, op_ins, op_outs);

            for op_in in op_ins {
                match outs.pop() {
                    Some(out) => self.expect_type(&out, &op_in, op.span),
                    None => ins.push(op_in),
                }
            }

            for op_out in op_outs {
                outs.push(op_out);
            }
        }

        let mut erased_ins = Vec::new();
        for block_in in ins {
            match block_in {
                TypeKind::Generic(index) => match self.erasures.get(index).unwrap() {
                    Some(type_kind) => erased_ins.push(type_kind.clone()),
                    None => erased_ins.push(block_in),
                },
                //TODO: this is going to need recursion
                TypeKind::List(element_type) => match *element_type {
                    TypeKind::Generic(index) => match self.erasures.get(index).unwrap() {
                        Some(type_kind) => {
                            erased_ins.push(TypeKind::List(Box::new(type_kind.clone())))
                        }
                        None => erased_ins.push(TypeKind::List(Box::new(*element_type))),
                    },
                    _ => erased_ins.push(*element_type),
                },
                _ => erased_ins.push(block_in),
            }
        }

        let mut erased_outs = Vec::new();
        for block_out in outs {
            match block_out {
                TypeKind::Generic(index) => match self.erasures.get(index).unwrap() {
                    Some(type_kind) => erased_outs.push(type_kind.clone()),
                    None => erased_outs.push(block_out),
                },
                //TODO: this is going to need recursion
                TypeKind::List(element_type) => match *element_type {
                    TypeKind::Generic(index) => match self.erasures.get(index).unwrap() {
                        Some(type_kind) => {
                            erased_outs.push(TypeKind::List(Box::new(type_kind.clone())))
                        }
                        None => erased_outs.push(TypeKind::List(Box::new(*element_type))),
                    },
                    _ => erased_outs.push(*element_type),
                },
                _ => erased_outs.push(block_out),
            }
        }
        (erased_ins, erased_outs)
    }
}
