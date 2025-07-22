use crate::lowerer::{ByteCodeInstruction, StackFrame};
use std::fs::File;
use std::io::Result;
use std::io::Write;

pub struct FasmEmitter {
    labels: usize, //TODO: this is just a massive hack to emit multiple newList ops
    out_file: File,
}

impl FasmEmitter {
    pub fn new(out_file: File) -> Self {
        FasmEmitter { labels: 0, out_file }
    }

    pub fn emit(
        &mut self,
        program: &[(String, StackFrame)],
        constants: &[String],
    ) -> Result<()> {
        self.emit_preamble()?;
        self.emit_helper_functions()?;

        for (name, frame) in program {
            writeln!(self.out_file, "{}:", name)?;
            if name == "main" {

                //subtract from rsp the number of locals
                writeln!(self.out_file, "push rbp")?;
                writeln!(self.out_file, "mov rbp, rsp")?;
                writeln!(self.out_file, "sub rsp, {}", frame.max_locals * 8)?;

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

            for op in &frame.instructions {
                if let ByteCodeInstruction::Return = op {
                    writeln!(self.out_file, "\tpop rax")?;
                    writeln!(self.out_file, "\tpop rbx")?; //restore volatile register
                }
                self.emit_op(op, constants)?;
            }

            if name == "main" {
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
        writeln!(self.out_file, "entry main")?;
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

    fn emit_op(&mut self, opcode: &ByteCodeInstruction, _constants: &[String]) -> Result<()> {
        writeln!(self.out_file, "; --- {:?} ---", opcode)?;
        match opcode {
            ByteCodeInstruction::Push(value) => writeln!(self.out_file, "\tpush {}", value),
            ByteCodeInstruction::NewList => {
                //Get length in words and in bytes
                writeln!(self.out_file, "\tpop r12")?; //r12 holds the list length
                writeln!(self.out_file, "\tmov r13, r12")?;
                writeln!(self.out_file, "\tinc r13")?; //r13 holds the list length + 1
                writeln!(self.out_file, "\tmov rcx, r13")?;
                writeln!(self.out_file, "\timul rcx, 8")?;

                //allocate memory for N+1 qwords
                writeln!(self.out_file, "\tsub rsp, 32")?;
                writeln!(self.out_file, "\tcall [malloc]")?;
                writeln!(self.out_file, "\tadd rsp, 32")?;

                //store pointer in rbx for now
                writeln!(self.out_file, "\tmov rbx, rax")?;

                //set length in element 0
                writeln!(self.out_file, "\tmov qword [rbx], r12")?;

                //set elements
                //organise loop
                writeln!(self.out_file, "\tmov rdx, 0")?;
                writeln!(self.out_file, ".loop_{}:", self.labels)?;
                let loop_label = self.labels;
                self.labels += 1;
                writeln!(self.out_file, "\tcmp rdx, r12")?;
                writeln!(self.out_file, "\tjge .end_{}", self.labels)?;
                let end_label = self.labels;
                self.labels += 1;

                //pop element i into rax
                writeln!(self.out_file, "\tpop rax")?;

                //increment counter before storing to place in the correct offset (0 is length)
                writeln!(self.out_file, "\tinc rdx")?;

                //store element
                writeln!(self.out_file, "\tmov qword [rbx + rdx*8], rax")?;

                writeln!(self.out_file, "\tjmp .loop_{}", loop_label)?;
                self.labels += 1;

                writeln!(self.out_file, ".end_{}:", end_label)?;
                //push pointer onto the stack
                writeln!(self.out_file, "\tpush rbx")

                //TODO: maybe have a refcount on the list, if it hits 0 free the memory
            }
            ByteCodeInstruction::Pop => writeln!(self.out_file, "\tpop rax"),
            ByteCodeInstruction::Dup => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Over => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpush rbx")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rbx")
            }
            ByteCodeInstruction::Rot => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpop rcx")?;
                writeln!(self.out_file, "\tpush rbx")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            ByteCodeInstruction::Swap => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rbx")
            }
            ByteCodeInstruction::Inc => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tinc rax")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Dec => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tdec rax")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Add => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tadd rax, rbx")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Sub => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tsub rbx, rax")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Mul => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\timul rax, rbx")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Div => {
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tcqo")?;
                writeln!(self.out_file, "\tidiv rbx")?;
                writeln!(self.out_file, "\tpush rdx")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Mod => {
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tcqo")?;
                writeln!(self.out_file, "\tidiv rbx")?;
                writeln!(self.out_file, "\tpush rax")?;
                writeln!(self.out_file, "\tpush rdx")
            }
            ByteCodeInstruction::Eq => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmove rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            ByteCodeInstruction::Gt => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmovg rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            ByteCodeInstruction::GtEq => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmovge rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            ByteCodeInstruction::Lt => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmovl rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            ByteCodeInstruction::LtEq => {
                writeln!(self.out_file, "\tmov rcx, 0")?;
                writeln!(self.out_file, "\tmov rdx, 1")?;
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tpop rbx")?;
                writeln!(self.out_file, "\tcmp rax, rbx")?;
                writeln!(self.out_file, "\tcmovle rcx, rdx")?;
                writeln!(self.out_file, "\tpush rcx")
            }
            ByteCodeInstruction::Print => {
                writeln!(self.out_file, "\tpop rcx")?;
                writeln!(self.out_file, "\tcall print_intln")
            }
            ByteCodeInstruction::PrintBool => {
                writeln!(self.out_file, "\tpop rcx")?;
                writeln!(self.out_file, "\tcall print_boolln")?;
                todo!("implement print_boolln")
            }
            ByteCodeInstruction::PrintList => {
                writeln!(self.out_file, "\tpop rcx")?;
                writeln!(self.out_file, "\tcall print_list")
            }

            ByteCodeInstruction::PushBlock { index } => {
                writeln!(self.out_file, "\tlea rax, [block_{}]", index)?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Load { index } => {
                writeln!(self.out_file, "\tmov rax, [rbp - {}]", index * 8)?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Store { index } => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tmov [rbp - {}], rax", index * 8)
            }
            ByteCodeInstruction::ListLen => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\tmov rax, [rax]")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::ListGet => {
                writeln!(self.out_file, "\tpop rax")?; //index
                writeln!(self.out_file, "\tinc rax")?; //index + 1
                writeln!(self.out_file, "\tpop rbx")?; //list
                writeln!(self.out_file, "\tmov rax, [rbx + rax*8]")?;
                writeln!(self.out_file, "\tpush rax")
            }
            ByteCodeInstruction::Label(label) => writeln!(self.out_file, ".label_{}:", label),
            ByteCodeInstruction::Call {
                in_count,
                out_count,
            } => {
                //Get pointer to function from the stack
                writeln!(self.out_file, "\tpop rax")?;

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

                writeln!(self.out_file, "\tcall rax")?;

                if *out_count == 1 {
                    writeln!(self.out_file, "\tpush rax")?;
                }
                Ok(())
            }
            ByteCodeInstruction::Jump { label } => writeln!(self.out_file, "\tjmp .label_{}", label),
            ByteCodeInstruction::JumpIfFalse { label } => {
                writeln!(self.out_file, "\tpop rax")?;
                writeln!(self.out_file, "\ttest rax, rax")?;
                writeln!(self.out_file, "\tjz .label_{}", label)
            }
            ByteCodeInstruction::Return => writeln!(self.out_file, "\tret"),
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
