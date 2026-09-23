use std::{
    fmt::Write as FmtWrite,
    fs::File,
    io::{self, Error, Write as IoWrite},
    path::Path,
};

use crate::enums::{Expression, NType, Program, Statement};

#[derive(Debug)]
pub struct FnBuffer {
    pub name: String,
    pub code: String,
}

#[derive(Debug)]
pub struct AsmCodegen {
    pub text_section: String,
    pub bss_section: String,
    pub data_section: String,
    pub rodata_section: String,

    pub main_buffer: String,

    pub func_buffers: Vec<FnBuffer>,

    pub label_counter: usize,
    pub exit_exists: bool,

    pub program: Program,
}

impl AsmCodegen {
    pub fn new(program: Program) -> Self {
        Self {
            text_section: String::new(),
            bss_section: String::new(),
            data_section: String::new(),
            rodata_section: String::new(),

            main_buffer: String::new(),
            func_buffers: Vec::new(),

            label_counter: 0,

            exit_exists: false,
            program,
        }
    }

    fn push_helper_exit_function(&mut self) {
        self.exit_exists = true;

        let fn_link = self.new_func("nnv_hlpr_exit_fn");
        fn_link.code.push_str("    mov rax, 60\n");
        fn_link.code.push_str("    syscall\n\n");
    }

    pub fn new_func(&mut self, name: &str) -> &mut FnBuffer {
        self.func_buffers.push(FnBuffer {
            name: name.to_string(),
            code: String::new(),
        });

        self.func_buffers.last_mut().unwrap()
    }

    pub fn add_string(&mut self, text: &str) -> String {
        let label = format!("str_{}", self.label_counter);
        self.label_counter += 1;

        self.rodata_section
            .push_str(&format!("    {}: db \"{}\", 0\n", label, text));
        label
    }

    pub fn generate_declaration(
        &mut self,
        name: &String,
        val: &Expression,
    ) -> Result<(), &'static str> {
        match val {
            Expression::Int(intv) => {
                self.data_section
                    .push_str(&format!("    {}: dq {}\n", name, intv));
            }
            Expression::Char(charv) => {
                self.data_section
                    .push_str(&format!("    {}: db '{}'\n", name, charv));
            }
            Expression::ConstChar(strv) => {
                self.data_section
                    .push_str(&format!("    {}: db \"{}\", 0\n", name, strv));
            }
            _ => {}
        }

        Ok(())
    }

    fn generate_exit(&mut self, code: usize, buffer: &mut String) -> Result<(), &'static str> {
        if !self.exit_exists {
            self.push_helper_exit_function();
        }

        writeln!(buffer, "    ; exit statement with code '{}'", code).unwrap();
        writeln!(buffer, "    mov rdi, {}", code).unwrap();
        writeln!(buffer, "    call nnv_hlpr_exit_fn\n").unwrap();

        Ok(())
    }

    fn generate_block(&mut self) {
        let statements = std::mem::take(&mut self.program.statements);

        for stmt in &statements {
            match stmt {
                Statement::Set { name, ty, val } => {
                    // Unwrap the value
                    let expr = val.as_ref().unwrap();
                    self.generate_declaration(&name, expr);
                }
                Statement::Exit { 0: code } => {
                    let coded = match *code {
                        Expression::Int(intv) => intv as usize,
                        Expression::Char(charv) => charv as usize,
                        _ => 0 as usize,
                    };
                    let mut buffer = std::mem::take(&mut self.main_buffer);

                    self.generate_exit(coded, &mut buffer);

                    self.main_buffer = buffer;
                }
                _ => {}
            }
        }

        self.program.statements = statements;
    }

    pub fn generate(&mut self) -> Result<(), &'static str> {
        self.generate_block();
        Ok(())
    }

    pub fn emitate_asm(&self) -> String {
        let mut final_asm = String::new();

        final_asm.push_str("global _start\n");

        if !self.rodata_section.is_empty() {
            final_asm.push_str("section .rodata\n");
            final_asm.push_str(&self.rodata_section);
            final_asm.push_str("\n");
        }

        if !self.bss_section.is_empty() {
            final_asm.push_str("section .bss\n");
            final_asm.push_str(&self.bss_section);
            final_asm.push_str("\n");
        }

        if !self.data_section.is_empty() {
            final_asm.push_str("section .data\n");
            final_asm.push_str(&self.data_section);
            final_asm.push_str("\n");
        }

        // Секция .text всегда прописывается
        final_asm.push_str("section .text\n");
        final_asm.push_str(&self.text_section);
        final_asm.push_str("\n");

        for fnc in &self.func_buffers {
            final_asm.push_str(&format!("{}:\n", fnc.name));

            final_asm.push_str("    push rbp\n");
            final_asm.push_str("    mov rbp, rsp\n\n");

            // Тело функции
            final_asm.push_str(&fnc.code);

            // Эпилог
            final_asm.push_str("    mov rsp, rbp\n");
            final_asm.push_str("    pop rbp\n");
            final_asm.push_str("    ret\n\n");
        }

        final_asm.push_str("_start:\n");
        final_asm.push_str(&self.main_buffer);

        final_asm.push_str("    mov rax, 60\n");
        final_asm.push_str("    xor rdi, rdi\n");
        final_asm.push_str("    syscall\n");

        final_asm
    }
}

pub fn write_asm(path: impl AsRef<Path>, code: String) -> io::Result<()> {
    if code.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Nothing to write. (Code is empty)",
        ));
    }

    let mut file = File::create(&path)?;
    file.write_all(code.as_bytes())?;

    println!("[*] Assembly written to {}", path.as_ref().display());

    Ok(())
}
