use crate::parser::{Op, OpKind};

#[derive(Debug, Clone)]
enum TypeKind {
    Bool,
    Int,
}

pub struct TypeChecker {
    type_stack: Vec<TypeKind>,
}

impl TypeChecker {
    pub fn new() -> TypeChecker {
        TypeChecker {
            type_stack: Vec::new(),
        }
    }

    pub fn type_check(&mut self, ops: &Vec<Op>) {
        for op in ops {
            match op.kind {
                OpKind::PushInt(_) => self.type_stack.push(TypeKind::Int),
                OpKind::Plus
                | OpKind::Minus
                | OpKind::Multiply
                | OpKind::Divide
                | OpKind::Modulo => match (self.type_stack.pop(), self.type_stack.pop()) {
                    (Some(TypeKind::Int), Some(TypeKind::Int)) => {
                        self.type_stack.push(TypeKind::Int)
                    }
                    (Some(a), Some(b)) => panic!("Expected [Int Int] but got [{:?} {:?}]", a, b),
                    _ => panic!("Expected [Int Int] but stack was empty"),
                },
                OpKind::Dup => match self.type_stack.pop() {
                    Some(a) => {
                        self.type_stack.push(a.clone());
                        self.type_stack.push(a);
                    }
                    None => panic!("Expected `Any` but stack was empty"),
                },
                OpKind::Swap => match self.type_stack.pop() {
                    Some(a) => match self.type_stack.pop() {
                        Some(b) => {
                            self.type_stack.push(a);
                            self.type_stack.push(b);
                        }
                        None => panic!("Expected `Any` but stack was empty"),
                    },
                    None => panic!("Expected `Any` but stack was empty"),
                },
                OpKind::Print => match self.type_stack.pop() {
                    Some(_) => (),
                    None => panic!("Expected `Any` but stack was empty"),
                },
            }
        }
    }
}
