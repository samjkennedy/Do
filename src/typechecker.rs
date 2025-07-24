use crate::diagnostic::Diagnostic;
use crate::lexer::{Span, TokenKind};
use crate::parser::{Op, OpKind};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::zip;

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

#[derive(Debug, Clone)]
pub enum TypedOpKind {
    PushBool(bool),
    PushInt(i64),
    PushList(Vec<TypedOp>),
    PushBlock(Vec<TypedOp>),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    Equals,
    Not,
    And,
    Or,
    Identity,
    Over,
    Pop,
    Rot,
    Swap,
    Dup,
    Print,
    Concat,
    Head,
    Tail,
    Push,
    Do,
    Filter,
    Fold,
    Foreach,
    Len,
    Map,
    DumpStack,
    DefineFunction {
        name: String,
        block: Box<TypedOp>,
    },
    Call(String),
    Binding {
        bindings: Vec<String>,
        body: Vec<TypedOp>,
    },
    Value(String),
    If {
        body: Vec<TypedOp>,
        else_body: Option<Vec<TypedOp>>,
    },
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

#[derive(Debug, Clone)]
pub struct TypedOp {
    pub kind: TypedOpKind,
    pub ins: Vec<TypeKind>,
    pub outs: Vec<TypeKind>, //No need for the outs in lowering yet, so comment it to silence the compiler warnings
}

#[derive(Clone)]
pub struct TypeChecker {
    fail_on_non_empty_stack: bool,
    pub type_stack: Vec<(TypeKind, Span)>,
    pub diagnostics: Vec<Diagnostic>,
    erasures: Vec<Option<TypeKind>>,
    next_generic_index: usize,
    functions: HashMap<String, (Vec<TypeKind>, Vec<TypeKind>)>,
    bindings: HashMap<String, TypeKind>,
    in_block: bool,
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
            bindings: HashMap::new(),
            in_block: false,
        }
    }

    pub fn type_check(&mut self, ops: &Vec<Op>) -> Vec<TypedOp> {
        let mut typed_ops = Vec::new();
        for op in ops {
            // println!("op: {:?}", op.kind);
            let typed_op = self.type_check_op(&op.kind, op.span);

            // println!("  ins: {:?}, outs: {:?}", ins, outs);
            self.resolve_type_stack(op, &typed_op);

            typed_ops.push(TypedOp {
                kind: typed_op.kind,
                ins: typed_op
                    .ins
                    .iter()
                    .map(|t| self.erase(t).unwrap_or(t.clone()))
                    .collect(),
                outs: typed_op
                    .outs
                    .iter()
                    .map(|t| self.erase(t).unwrap_or(t.clone()))
                    .collect(),
            });
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
        typed_ops
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
                    None => self.erase_generic(index, actual),
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

    fn type_check_op(&mut self, op_kind: &OpKind, span: Span) -> TypedOp {
        match op_kind {
            OpKind::PushBool(value) => TypedOp {
                kind: TypedOpKind::PushBool(*value),
                ins: vec![],
                outs: vec![TypeKind::Bool],
            },
            OpKind::PushInt(value) => TypedOp {
                kind: TypedOpKind::PushInt(*value),
                ins: vec![],
                outs: vec![TypeKind::Int],
            },
            OpKind::PushList(ops) => {
                let mut element_type: Option<TypeKind> = None;

                let mut elements = Vec::new();
                for op in ops {
                    let typed_op = self.type_check_op(&op.kind, span);
                    if !typed_op.ins.is_empty() || typed_op.outs.len() != 1 {
                        self.diagnostics.push(Diagnostic::report_error(
                            format!(
                                "List elements must have the signature [ -- <a>], got [{} -- {}]",
                                typed_op
                                    .ins
                                    .iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join(" "),
                                typed_op
                                    .outs
                                    .iter()
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
                                TypedOp {
                                    kind: TypedOpKind::PushList(vec![]),
                                    ins: vec![],
                                    outs: vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                                }
                            }
                            Some(type_kind) => TypedOp {
                                kind: TypedOpKind::PushList(vec![]),
                                ins: vec![],
                                outs: vec![TypeKind::List(Box::new(type_kind))],
                            },
                        };
                    }

                    let out = typed_op.outs.first().unwrap();
                    match &element_type {
                        Some(type_kind) => self.expect_type(type_kind, out, op.span),
                        None => element_type = Some(out.clone()),
                    }

                    elements.push(typed_op);
                }
                match element_type {
                    None => {
                        let index = self.create_generic();
                        TypedOp {
                            kind: TypedOpKind::PushList(elements),
                            ins: vec![],
                            outs: vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                        }
                    }
                    Some(type_kind) => TypedOp {
                        kind: TypedOpKind::PushList(elements),
                        ins: vec![],
                        outs: vec![TypeKind::List(Box::new(type_kind))],
                    },
                }
            }
            OpKind::PushFunction(ops) => {
                let typed_block = self.type_check_block(ops, span);
                TypedOp {
                    kind: typed_block.kind,
                    ins: vec![],
                    outs: vec![TypeKind::Block {
                        ins: typed_block.ins,
                        outs: typed_block.outs,
                    }],
                }
            }

            OpKind::Plus | OpKind::Minus | OpKind::Multiply | OpKind::Divide | OpKind::Modulo => {
                TypedOp {
                    kind: match op_kind {
                        OpKind::Plus => TypedOpKind::Plus,
                        OpKind::Minus => TypedOpKind::Minus,
                        OpKind::Multiply => TypedOpKind::Multiply,
                        OpKind::Divide => TypedOpKind::Divide,
                        OpKind::Modulo => TypedOpKind::Modulo,
                        _ => unreachable!(),
                    },
                    ins: vec![TypeKind::Int, TypeKind::Int],
                    outs: vec![TypeKind::Int],
                }
            }
            OpKind::LessThan
            | OpKind::GreaterThan
            | OpKind::LessThanEquals
            | OpKind::GreaterThanEquals => TypedOp {
                kind: match op_kind {
                    OpKind::LessThan => TypedOpKind::LessThan,
                    OpKind::GreaterThan => TypedOpKind::GreaterThan,
                    OpKind::LessThanEquals => TypedOpKind::LessThanEquals,
                    OpKind::GreaterThanEquals => TypedOpKind::GreaterThanEquals,
                    _ => unreachable!(),
                },
                ins: vec![TypeKind::Int, TypeKind::Int],
                outs: vec![TypeKind::Bool],
            },
            OpKind::Equals => {
                let index = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Equals,
                    ins: vec![TypeKind::Generic(index), TypeKind::Generic(index)],
                    outs: vec![TypeKind::Bool],
                }
            }
            OpKind::Not => TypedOp {
                kind: TypedOpKind::Not,
                ins: vec![TypeKind::Bool],
                outs: vec![TypeKind::Bool],
            },
            OpKind::Identity => {
                let index = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Identity,
                    ins: vec![TypeKind::Generic(index)],
                    outs: vec![TypeKind::Generic(index)],
                }
            }
            OpKind::And => TypedOp {
                kind: TypedOpKind::And,
                ins: vec![TypeKind::Bool],
                outs: vec![TypeKind::Bool],
            },
            OpKind::Or => TypedOp {
                kind: TypedOpKind::Or,
                ins: vec![TypeKind::Bool],
                outs: vec![TypeKind::Bool],
            },
            OpKind::Dup => {
                let index = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Dup,
                    ins: vec![TypeKind::Generic(index)],
                    outs: vec![TypeKind::Generic(index), TypeKind::Generic(index)],
                }
            }
            OpKind::Len => {
                let index = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Len,
                    ins: vec![TypeKind::Generic(index)],
                    outs: vec![TypeKind::Int],
                }
            }
            OpKind::Over => {
                let a = self.create_generic();
                let b = self.create_generic();

                TypedOp {
                    kind: TypedOpKind::Over,
                    ins: vec![TypeKind::Generic(a), TypeKind::Generic(b)],
                    outs: vec![
                        TypeKind::Generic(a),
                        TypeKind::Generic(b),
                        TypeKind::Generic(a),
                    ],
                }
            }
            OpKind::Pop => {
                let index = self.create_generic();

                TypedOp {
                    kind: TypedOpKind::Pop,
                    ins: vec![TypeKind::Generic(index)],
                    outs: vec![],
                }
            }
            OpKind::Rot => {
                let a = self.create_generic();
                let b = self.create_generic();
                let c = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Rot,
                    ins: vec![
                        TypeKind::Generic(a),
                        TypeKind::Generic(b),
                        TypeKind::Generic(c),
                    ],
                    outs: vec![
                        TypeKind::Generic(b),
                        TypeKind::Generic(a),
                        TypeKind::Generic(c),
                    ],
                }
            }
            OpKind::Swap => {
                let a = self.create_generic();
                let b = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Swap,
                    ins: vec![TypeKind::Generic(a), TypeKind::Generic(b)],
                    outs: vec![TypeKind::Generic(a), TypeKind::Generic(b)],
                }
            }
            OpKind::Print => {
                let index = self.create_generic();

                TypedOp {
                    kind: TypedOpKind::Print,
                    ins: vec![TypeKind::Generic(index)],
                    outs: vec![],
                }
            }
            OpKind::Concat => {
                let index = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Concat,
                    ins: vec![
                        TypeKind::List(Box::new(TypeKind::Generic(index))),
                        TypeKind::List(Box::new(TypeKind::Generic(index))),
                    ],
                    outs: vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                }
            }
            OpKind::Push => {
                let index = self.create_generic();

                TypedOp {
                    kind: TypedOpKind::Push,
                    ins: vec![
                        TypeKind::Generic(index),
                        TypeKind::List(Box::new(TypeKind::Generic(index))),
                    ],
                    outs: vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                }
            }
            OpKind::Head => {
                let index = self.create_generic();

                TypedOp {
                    kind: TypedOpKind::Head,
                    ins: vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                    outs: vec![TypeKind::Generic(index)],
                }
            }
            OpKind::Tail => {
                let index = self.create_generic();

                TypedOp {
                    kind: TypedOpKind::Tail,
                    ins: vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                    outs: vec![TypeKind::List(Box::new(TypeKind::Generic(index)))],
                }
            }
            OpKind::Do => TypedOp {
                kind: TypedOpKind::Do,
                ins: vec![TypeKind::Block {
                    ins: vec![], //TODO: Do should accept varargs
                    outs: vec![],
                }],
                outs: vec![],
            },
            OpKind::Filter => {
                let a = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Filter,
                    ins: vec![
                        TypeKind::Block {
                            ins: vec![TypeKind::Generic(a)],
                            outs: vec![TypeKind::Bool],
                        },
                        TypeKind::List(Box::new(TypeKind::Generic(a))),
                    ],
                    outs: vec![TypeKind::List(Box::new(TypeKind::Generic(a)))],
                }
            }
            OpKind::Fold => {
                let a = self.create_generic();
                let b = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Fold,
                    ins: vec![
                        TypeKind::Generic(b),
                        TypeKind::Block {
                            ins: vec![TypeKind::Generic(a), TypeKind::Generic(b)],
                            outs: vec![TypeKind::Generic(b)],
                        },
                        TypeKind::List(Box::new(TypeKind::Generic(a))),
                    ],
                    outs: vec![TypeKind::Generic(b)],
                }
            }
            OpKind::Foreach => {
                let a = self.create_generic();
                TypedOp {
                    kind: TypedOpKind::Foreach,
                    ins: vec![
                        TypeKind::Block {
                            ins: vec![TypeKind::Generic(a)],
                            outs: vec![],
                        },
                        TypeKind::List(Box::new(TypeKind::Generic(a))),
                    ],
                    outs: vec![],
                }
            }
            OpKind::Map => {
                let a = self.create_generic();
                let b = self.create_generic();

                TypedOp {
                    kind: TypedOpKind::Map,
                    ins: vec![
                        TypeKind::Block {
                            ins: vec![TypeKind::Generic(a)],
                            outs: vec![TypeKind::Generic(b)],
                        },
                        TypeKind::List(Box::new(TypeKind::Generic(a))),
                    ],
                    outs: vec![TypeKind::List(Box::new(TypeKind::Generic(b)))],
                }
            }
            OpKind::DumpStack => {
                for (type_kind, span) in &self.type_stack {
                    println!(
                        "{} defined at {:?}",
                        self.erase(type_kind).unwrap_or(type_kind.clone()),
                        span
                    );
                }
                TypedOp {
                    kind: TypedOpKind::DumpStack,
                    ins: vec![],
                    outs: vec![],
                }
            }
            OpKind::DefineFunction { identifier, body } => {
                if let TokenKind::Identifier(name) = &identifier.kind {
                    if let OpKind::PushFunction(ops) = &body.kind {
                        let block = self.type_check_block(ops, span);

                        self.functions
                            .insert(name.clone(), (block.ins.clone(), block.outs.clone()));

                        TypedOp {
                            kind: TypedOpKind::DefineFunction {
                                name: name.clone(),
                                block: Box::new(block.clone()),
                            },
                            //declaring a function doesn't affect the stack
                            ins: vec![],
                            outs: vec![],
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            OpKind::Identifier(name) => {
                match self.bindings.get(name) {
                    Some(type_kind) => TypedOp {
                        kind: TypedOpKind::Value(name.clone()),
                        ins: vec![],
                        outs: vec![type_kind.clone()],
                    },
                    None => match self.functions.get(name) {
                        Some((ins, outs)) => TypedOp {
                            kind: TypedOpKind::Call(name.clone()),
                            ins: ins.clone(),
                            outs: outs.clone(),
                        },
                        None => {
                            self.diagnostics.push(Diagnostic::report_error(
                                format!("no such identifier `{}` in scope", name),
                                span,
                            ));
                            //return bogus to keep going
                            TypedOp {
                                kind: TypedOpKind::Call(name.clone()),
                                ins: vec![],
                                outs: vec![],
                            }
                        }
                    },
                }
            }
            OpKind::If { body, else_body } => match else_body {
                Some(else_body) => match self.pop_type(span) {
                    Some((TypeKind::Bool, bool_span)) => {
                        let mut typed_ops = Vec::new();
                        for op in body {
                            let typed_op = self.type_check_op(&op.kind, op.span);

                            self.resolve_type_stack(op, &typed_op);

                            typed_ops.push(typed_op);
                        }
                        let checked_body = self.type_check_block(body, span);
                        self.type_stack.push((TypeKind::Bool, bool_span));
                        
                        let checked_else_body = self.type_check_block(else_body, span);

                        self.check_op_symmetrical(span, &checked_body);
                        self.check_op_symmetrical(span, &checked_else_body);

                        for (body_in, else_in) in zip(&checked_body.ins, &checked_else_body.ins) {
                            self.expect_type(body_in, else_in, span);
                        }
                        for (body_out, else_out) in zip(&checked_body.outs, &checked_else_body.outs)
                        {
                            self.expect_type(body_out, else_out, span);
                        }

                        if let TypedOpKind::PushBlock(typed_body_ops) = &checked_body.kind {
                            if let TypedOpKind::PushBlock(typed_else_body_ops) =
                                &checked_else_body.kind
                            {

                                let mut ins = vec![TypeKind::Bool];
                                ins.extend(checked_body.ins);
                                TypedOp {
                                    ins,
                                    outs: checked_body.outs,
                                    kind: TypedOpKind::If {
                                        body: typed_body_ops.clone(),
                                        else_body: Some(typed_else_body_ops.clone()),
                                    },
                                }
                            } else {
                                unreachable!()
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    Some((type_kind, span)) => {
                        self.diagnostics.push(Diagnostic::report_error(
                            format!("expected {} but got {}", TypeKind::Bool, type_kind),
                            span,
                        ));
                        TypedOp {
                            ins: vec![TypeKind::Bool],
                            outs: vec![],
                            kind: TypedOpKind::If {
                                body: vec![],
                                else_body: Some(vec![]),
                            },
                        }
                    }
                    None => {
                        self.diagnostics.push(Diagnostic::report_error(
                            format!("expected {} but stack was empty", TypeKind::Bool),
                            span,
                        ));
                        TypedOp {
                            ins: vec![TypeKind::Bool],
                            outs: vec![],
                            kind: TypedOpKind::If {
                                body: vec![],
                                else_body: Some(vec![]),
                            },
                        }
                    }
                },
                None => match self.pop_type(span) {
                    Some((TypeKind::Bool, bool_span)) => {
                        let mut typed_ops = Vec::new();
                        for op in body {
                            let typed_op = self.type_check_op(&op.kind, op.span);

                            self.resolve_type_stack(op, &typed_op);

                            typed_ops.push(typed_op);
                        }
                        let checked_body = self.type_check_block(body, span);
                        self.check_op_symmetrical(span, &checked_body);

                        if let TypedOpKind::PushBlock(typed_ops) = &checked_body.kind {
                            self.type_stack.push((TypeKind::Bool, bool_span));

                            let mut ins = vec![TypeKind::Bool];
                            ins.extend(checked_body.ins);
                            TypedOp {
                                ins,
                                outs: checked_body.outs,
                                kind: TypedOpKind::If {
                                    body: typed_ops.clone(),
                                    else_body: None,
                                },
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    Some((type_kind, span)) => {
                        self.diagnostics.push(Diagnostic::report_error(
                            format!("expected {} but got {}", TypeKind::Bool, type_kind),
                            span,
                        ));
                        TypedOp {
                            ins: vec![TypeKind::Bool],
                            outs: vec![],
                            kind: TypedOpKind::If {
                                body: vec![],
                                else_body: None,
                            },
                        }
                    }
                    None => {
                        self.diagnostics.push(Diagnostic::report_error(
                            format!("expected {} but stack was empty", TypeKind::Bool),
                            span,
                        ));
                        TypedOp {
                            ins: vec![TypeKind::Bool],
                            outs: vec![],
                            kind: TypedOpKind::If {
                                body: vec![],
                                else_body: None,
                            },
                        }
                    }
                },
            },
            OpKind::Binding { bindings, body } => {
                let mut binding_identifiers = Vec::new();
                for identifier in bindings.iter().rev() {
                    if let TokenKind::Identifier(name) = &identifier.kind {
                        match self.pop_type(identifier.span) {
                            Some((type_kind, _)) => {
                                self.bindings.insert(name.clone(), type_kind);
                                binding_identifiers.push(name.clone());
                            }
                            None => break,
                        }
                    } else {
                        unreachable!()
                    }
                }

                if let OpKind::PushFunction(ops) = &body.kind {
                    let mut typed_ops = Vec::new();

                    for op in ops {
                        let typed_op = self.type_check_op(&op.kind, op.span);

                        self.resolve_type_stack(op, &typed_op);

                        typed_ops.push(typed_op);
                    }

                    TypedOp {
                        ins: vec![],
                        outs: vec![],
                        kind: TypedOpKind::Binding {
                            bindings: binding_identifiers,
                            body: typed_ops,
                        },
                    }
                } else {
                    unreachable!()
                }
            }
        }
    }

    fn check_op_symmetrical(&mut self, span: Span, op: &TypedOp) {
        if op.ins.len() != op.outs.len() {
            self.diagnostics.push(Diagnostic::report_error(
                format!(
                    "expected symmetrical function, but got [{} -- {}]",
                    op.ins
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                    op.outs
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                ),
                span,
            ))
        }

        for (block_in, block_out) in zip(&op.ins, &op.outs) {
            if block_in != block_out {
                self.diagnostics.push(Diagnostic::report_error(
                    format!(
                        "expected symmetrical function, but got [{} -- {}]",
                        op.ins
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                        op.outs
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(" ")
                    ),
                    span,
                ))
            }
        }
    }

    fn resolve_type_stack(&mut self, op: &Op, typed_op: &TypedOp) {
        for input in typed_op.ins.clone() {
            match self.type_stack.pop() {
                Some((type_kind, span)) => self.expect_type(&type_kind, &input, op.span),
                None => self.diagnostics.push(Diagnostic::report_error(
                    format!("Expected {} but stack was empty", input),
                    op.span,
                )),
            }
        }

        for output in typed_op.outs.clone() {
            self.type_stack.push((output, op.span));
        }
    }

    fn pop_type(&mut self, span: Span) -> Option<(TypeKind, Span)> {
        match self.type_stack.pop() {
            Some((type_kind, span)) => Some((type_kind, span)),
            None => {
                if self.in_block {
                    let generic = self.create_generic();
                    Some((TypeKind::Generic(generic), span))
                } else {
                    self.diagnostics.push(Diagnostic::report_error(
                        "expected value but stack was empty".to_string(),
                        span,
                    ));
                    None
                }
            }
        }
    }

    fn type_check_block(&mut self, ops: &Vec<Op>, span: Span) -> TypedOp {
        let mut typed_ops = Vec::new();
        let mut ins: Vec<TypeKind> = Vec::new();
        let mut outs: Vec<TypeKind> = Vec::new();

        let was_in_block = self.in_block;
        self.in_block = true;

        for op in ops {
            let typed_op = self.type_check_op(&op.kind, span);

            // println!("  op: {:?}, op_ins: {:?}, op_outs: {:?}", op, op_ins, op_outs);

            for op_in in &typed_op.ins {
                match outs.pop() {
                    Some(out) => self.expect_type(&out, op_in, op.span),
                    None => ins.push(op_in.clone()),
                }
            }

            for op_out in &typed_op.outs {
                outs.push(op_out.clone());
            }
            typed_ops.push(typed_op);
        }

        self.in_block = was_in_block;

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
        TypedOp {
            kind: TypedOpKind::PushBlock(typed_ops),
            ins: erased_ins,
            outs: erased_outs,
        }
    }
}
