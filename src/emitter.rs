use crate::lowerer::OpCode;
use std::fs::File;
use std::io::Result;
use std::io::Write;

pub struct FasmEmitter {
    out_file: File,
}

impl FasmEmitter {
    pub fn new(out_file: File) -> Self {
        FasmEmitter { out_file }
    }

    pub fn emit(&mut self, program: &[(String, Vec<OpCode>)], constants: &[String]) -> Result<()> {
        self.emit_preamble()?;

        self.emit_helper_functions()?;
        self.emit_built_in_functions()?;

        for (name, bytecode) in program {
            writeln!(self.out_file, "{}:", name)?;
            if name == "start" {
                writeln!(
                    self.out_file,
                    "sub rsp, 8 ; align the stack to 16 bytes (Windows ABI requirement)"
                )?;
            } else {
                writeln!(self.out_file, "\tpush rbx")?; //preserve volatile register

                //TODO: needs the signature to know how many to push
                //      or fully just use the stack
                writeln!(self.out_file, "\tpush rcx")?;
            }

            for op in bytecode {
                if let OpCode::Return = op {
                    writeln!(self.out_file, "\tpop rax")?;
                    writeln!(self.out_file, "\tpop rbx")?; //restore volatile register
                }
                self.emit_op(op, constants)?;
            }

            if name == "start" {
                writeln!(self.out_file, "; --- return ---")?;
                writeln!(self.out_file, "\txor ecx, ecx")?;
                writeln!(self.out_file, "\tcall [ExitProcess]")?;
            }
        }

        self.emit_prologue()?;

        self.out_file.flush()?;

        Ok(())
    }

    fn emit_preamble(&mut self) -> Result<()> {
        writeln!(self.out_file, "format PE64 console")?;
        writeln!(self.out_file, "entry start")?;
        writeln!(self.out_file)?;
        writeln!(self.out_file, "include 'win64a.inc'")?;
        writeln!(self.out_file)?;
        writeln!(self.out_file, "section '.text' code readable executable")?;
        writeln!(self.out_file)?;
        Ok(())
    }

    fn emit_helper_functions(&mut self) -> Result<()> {
        self.emit_print_intln_function()?;
        self.emit_print_int_function()?;
        self.emit_print_list_function()?;
        Ok(())
    }

    fn emit_built_in_functions(&mut self) -> Result<()> {
        self.emit_map_function()?;
        Ok(())
    }

    fn emit_print_intln_function(&mut self) -> Result<()> {
        writeln!(self.out_file, "print_intln:")?;
        writeln!(self.out_file, "\tsub rsp, 32; shadow space for Win-x64 ABI")?;
        writeln!(self.out_file, "\tmov rdx, rcx")?;
        writeln!(self.out_file, "\tlea rcx, [fmt_intln]")?;
        writeln!(self.out_file, "\tcall [printf]")?;
        writeln!(self.out_file, "\tadd rsp, 32; clean shadow space")?;
        writeln!(self.out_file, "\tret")?;
        Ok(())
    }

    fn emit_print_int_function(&mut self) -> Result<()> {
        writeln!(self.out_file, "print_int:")?;
        writeln!(self.out_file, "\tsub rsp, 32; shadow space for Win-x64 ABI")?;
        writeln!(self.out_file, "\tmov rdx, rcx")?;
        writeln!(self.out_file, "\tlea rcx, [fmt_int]")?;
        writeln!(self.out_file, "\tcall [printf]")?;
        writeln!(self.out_file, "\tadd rsp, 32; clean shadow space")?;
        writeln!(self.out_file, "\tret")?;
        Ok(())
    }

    fn emit_print_list_function(&mut self) -> Result<()> {
        writeln!(self.out_file, "print_list:")?;

        //push the pointer to the list into rsi
        writeln!(self.out_file, "\tmov rsi, rcx")?;

        writeln!(self.out_file, "; print opening '['")?;
        writeln!(self.out_file, "\tsub rsp, 32")?;
        writeln!(self.out_file, "\tlea rcx, [fmt_str]")?;
        writeln!(self.out_file, "\tlea rdx, [lbracket]")?;
        writeln!(self.out_file, "\tcall [printf]")?;
        writeln!(self.out_file, "\tadd rsp, 32")?;

        writeln!(self.out_file, "; prepare loop")?;
        //r12-r14 are non volatile
        writeln!(self.out_file, "\tmov r12, 0")?; //r12 holds the loop counter
        writeln!(self.out_file, "\tmov r13, [rsi]")?; //r13 holds the list length
        writeln!(self.out_file, "\tlea r14, [rsi + 8]")?; //r14 holds the base of values

        //loop
        writeln!(self.out_file, ".loop:")?;
        //print element
        writeln!(self.out_file, "; print element")?;
        //TODO: this will fail spectacularly for nested lists
        writeln!(self.out_file, "\tmov rcx, [r14 + r12*8]")?;
        writeln!(self.out_file, "\tcall print_int")?;

        writeln!(self.out_file, "; exit loop if last")?;
        writeln!(self.out_file, "\tinc r12")?;
        writeln!(self.out_file, "\tcmp r12, r13")?;
        writeln!(self.out_file, "\tjge .done")?;

        writeln!(self.out_file, "; print space")?;
        writeln!(self.out_file, "\tsub rsp, 32")?;
        writeln!(self.out_file, "\tlea rcx, [fmt_str]")?;
        writeln!(self.out_file, "\tlea rdx, [space]")?;
        writeln!(self.out_file, "\tcall [printf]")?;
        writeln!(self.out_file, "\tadd rsp, 32")?;
        writeln!(self.out_file, "\tjmp .loop")?;

        writeln!(self.out_file, ".done:")?;
        writeln!(self.out_file, "; print closing ']'")?;
        writeln!(self.out_file, "\tsub rsp, 32")?;
        writeln!(self.out_file, "\tlea rcx, [fmt_str]")?;
        writeln!(self.out_file, "\tlea rdx, [rbracket]")?;
        writeln!(self.out_file, "\tcall [printf]")?;
        writeln!(self.out_file, "\tadd rsp, 32")?;

        writeln!(self.out_file, "\txor ecx, ecx")?;
        writeln!(self.out_file, "\tret")?;
        writeln!(self.out_file)?;
        Ok(())
    }

    fn emit_map_function(&mut self) -> Result<()> {
        writeln!(self.out_file, "map:")?;
        writeln!(self.out_file, "; rcx = function pointer")?;
        writeln!(self.out_file, "; rdx = input list")?;
        writeln!(
            self.out_file,
            "\tmov r15, rcx            ; r15 = function pointer"
        )?;
        writeln!(
            self.out_file,
            "\tmov rsi, rdx            ; rsi = input list"
        )?;
        writeln!(
            self.out_file,
            "\tmov r13, [rsi]          ; r13 = number of elements"
        )?;
        writeln!(
            self.out_file,
            "\tlea r14, [rsi + 8]      ; r14 = pointer to original list elements"
        )?;
        writeln!(
            self.out_file,
            "; Allocate (length + 1) * 8 bytes (for length + elements)"
        )?;
        writeln!(self.out_file, "\tmov rax, r13")?;
        writeln!(
            self.out_file,
            "\tinc rax                 ; rax = length + 1"
        )?;
        writeln!(
            self.out_file,
            "\timul rax, 8             ; bytes = (length + 1) * 8"
        )?;
        writeln!(self.out_file, "\tsub rsp, 32             ; shadow space")?;
        writeln!(
            self.out_file,
            "\tmov rcx, rax            ; malloc(size) → rcx"
        )?;
        writeln!(self.out_file, "\tcall [malloc]")?;
        writeln!(self.out_file, "\tadd rsp, 32")?;
        writeln!(
            self.out_file,
            "\tmov rbx, rax            ; rbx = new list pointer"
        )?;
        writeln!(
            self.out_file,
            "\tmov [rbx], r13          ; store length at offset 0"
        )?;
        writeln!(
            self.out_file,
            "\tlea rdx, [rbx + 8]      ; rdx = ptr to new list elements"
        )?;
        writeln!(self.out_file, "\txor r12, r12            ; r12 = index")?;
        writeln!(self.out_file, ".loop:")?;
        writeln!(self.out_file, "\tcmp r12, r13")?;
        writeln!(self.out_file, "\tjge .done")?;
        writeln!(
            self.out_file,
            "\tmov rcx, [r14 + r12*8]  ; rcx = original[i]"
        )?;
        writeln!(
            self.out_file,
            "\tcall r15                ; apply function, result in rax"
        )?;
        writeln!(self.out_file, "\tmov [rdx + r12*8], rax  ; new[i] = result")?;
        writeln!(self.out_file, "\tinc r12")?;
        writeln!(self.out_file, "\tjmp .loop")?;
        writeln!(self.out_file, ".done:")?;
        writeln!(self.out_file, "\tmov rax, rbx            ; return new list")?;
        writeln!(self.out_file, "\tret")
    }

    fn emit_op(&mut self, opcode: &OpCode, constants: &[String]) -> Result<()> {
        writeln!(self.out_file, "; --- {:?} ---", opcode)?;
        match opcode {
            OpCode::Push(value) => writeln!(self.out_file, "\tpush {}", value),
            OpCode::NewList { length } => {
                //allocate memory for N+1 qwords
                writeln!(self.out_file, "\tsub rsp, 32")?;
                writeln!(self.out_file, "\tmov rcx, {}", (length + 1) * 8)?;
                writeln!(self.out_file, "\tcall [malloc]")?;
                writeln!(self.out_file, "\tadd rsp, 32")?;
                //store pointer in rbx for now
                writeln!(self.out_file, "\tmov rbx, rax")?;

                //set length in element 0
                writeln!(self.out_file, "\tmov qword [rbx], {}", length)?;

                //set elements
                for i in 0..*length {
                    //pop element i into rax
                    writeln!(self.out_file, "\tpop rax")?;
                    writeln!(self.out_file, "\tmov qword [rbx + {}], rax", (i + 1) * 8)?;
                }
                //push pointer onto the stack
                writeln!(self.out_file, "\tpush rbx")

                //TODO: maybe have a refcount on the list, if it hits 0 free the memory
            }
            OpCode::Pop => writeln!(self.out_file, "\tpop rax"),
            OpCode::Dup => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rax")
            }
            OpCode::Over => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpush rbx")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rbx")
            }
            OpCode::Rot => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpop rcx")?;
                writeln!(self.out_file, "\tpush rbx")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            OpCode::Swap => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rbx")
            }
            OpCode::Add => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tadd rax, rbx")?;
                writeln!(self.out_file, "\tpush rax")
            }
            OpCode::Sub => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tsub rbx, rax")?;
                writeln!(self.out_file, "\tpush rax")
            }
            OpCode::Mul => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\timul rax, rbx")?;
                writeln!(self.out_file, "\tpush rax")
            }
            OpCode::Div => {
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tcqo")?;
                writeln!(self.out_file, "\tidiv rbx")?;
                writeln!(self.out_file, "\tpush rdx")?;
                writeln!(self.out_file, "\tpush rax")
            }
            OpCode::Mod => {
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tcqo")?;
                writeln!(self.out_file, "\tidiv rbx")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rdx")
            }
            OpCode::Eq => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmove rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            OpCode::Gt => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmovg rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            OpCode::GtEq => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmovge rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            OpCode::Lt => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmovl rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            OpCode::LtEq => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmovle rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            OpCode::Print => {
                writeln!(self.out_file, "\tpop rcx")?;
                writeln!(self.out_file, "\tcall print_intln")
            }
            OpCode::PrintList => {
                writeln!(self.out_file, "\tpop rcx")?;
                writeln!(self.out_file, "\tcall print_list")
            }

            OpCode::PushBlock { index } => {
                writeln!(self.out_file, "\tlea rax, [block_{}]", index)?;
                writeln!(self.out_file, "\tpush rax ; function pointer")
            }
            OpCode::Load { index } => todo!(),
            OpCode::Store { index } => todo!(),
            OpCode::ListLen => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tmov rax, [rax]")?;
                writeln!(self.out_file, "\tpush rax")
            }
            OpCode::ListGet => {
                writeln!(self.out_file, "\tpop rax")?; //index
                writeln!(self.out_file, "\tinc rax")?; //index+1
                writeln!(self.out_file, "\tpop rbx")?; //list
                writeln!(self.out_file, "\tmov rax, [rbx + rax*8]")?;
                writeln!(self.out_file, "\tpush rax")
            }
            OpCode::ListSet => {
                writeln!(self.out_file, "\tpop rax")?; //value
                writeln!(self.out_file, "\tpop rbx")?; //index
                writeln!(self.out_file, "\tinc rbx")?; //index+1
                writeln!(self.out_file, "\tpop rcx")?; //list
                writeln!(self.out_file, "\tmov qword [rcx + rbx*8], rax")
            }
            OpCode::Label(label) => writeln!(self.out_file, ".label_{}", label),
            OpCode::Call {
                index,
                in_count,
                out_count,
            } => {
                let in_regs = ["rcx", "rdx", "r8", "r9"];
                if *in_count > 4 {
                    todo!("more than 4 ins")
                }
                if *out_count > 1 {
                    todo!("more than 1 outs")
                }
                for reg in in_regs.iter().take(*in_count) {
                    writeln!(self.out_file, "\tpop {}", reg)?;
                }

                let constant = constants.get(*index).unwrap();
                writeln!(self.out_file, "\tcall {}", constant)?;

                if *out_count == 1 {
                    writeln!(self.out_file, "\tpush rax")?;
                }
                Ok(())
            }
            OpCode::Jump { label } => writeln!(self.out_file, "\tjmp label_{}", label),
            OpCode::JumpIfFalse { label } => writeln!(self.out_file, "\tjz label_{}", label),
            OpCode::Map => {
                writeln!(self.out_file, "\tpop rcx")?; //pop func into rcx
                writeln!(self.out_file, "\tpop rdx")?; //pop list into rdx
                writeln!(self.out_file, "\tcall map")?;
                writeln!(self.out_file, "\tpush rax") //push resulting list onto the stack
            }
            OpCode::Return => writeln!(self.out_file, "\tret"),
        }
    }

    fn emit_prologue(&mut self) -> Result<()> {
        writeln!(self.out_file)?;
        writeln!(self.out_file, "section '.data' data readable writeable")?;
        writeln!(self.out_file, "fmt_intln db \"%lld\",10, 0")?;
        writeln!(self.out_file, "fmt_int   db \"%lld\",    0")?;
        writeln!(self.out_file, "fmt_str   db \"%s\",      0")?;
        writeln!(self.out_file, "lbracket  db \"[\",       0")?;
        writeln!(self.out_file, "space     db \" \",       0")?;
        writeln!(self.out_file, "rbracket  db \"]\",10,    0")?;
        writeln!(self.out_file)?;

        writeln!(self.out_file, "section '.idata' import data readable")?;
        writeln!(self.out_file)?;
        writeln!(
            self.out_file,
            "library kernel32, 'kernel32.dll', msvcrt, 'msvcrt.dll'"
        )?;
        writeln!(self.out_file, "import kernel32, ExitProcess, 'ExitProcess'")?;
        writeln!(
            self.out_file,
            "import msvcrt, printf, 'printf', malloc, 'malloc'"
        )?;
        Ok(())
    }
}
