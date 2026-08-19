use std::{fmt::format, fs, process::Command};

use crate::{Expression, NType, Op, Program, Statement, warning};

pub struct Codegen {
    version: String,
    rep: String,
    program: Program,
    generated: String,
    warnings: u32,
}

impl Codegen {
    pub fn new(ast: Program) -> Self {
        Self {
            version: String::from("0.0.1"),
            rep: String::from("-rc0.1"),
            program: ast,
            generated: String::new(),
            warnings: 0,
        }
    }

    fn generate_if(&self, expr: &Expression) {

    }

    pub fn generate_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Int(val) => val.to_string(),
            Expression::Var(name) => name.clone(),
            Expression::Char(c) => c.to_string(),
            Expression::BinaryOp { left, op, right } => {
                let left_code = self.generate_expression(left);
                let right_code = self.generate_expression(right);

                let op_s = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Equal => "=",
                    Op::Greater => ">",
                    Op::Less => "<",
                    Op::GreaterEq => ">=",
                    Op::LessEq => "<=",
                    Op::NotEq => "!=",

                    _ => todo!("Ops"),
                };

                format!("({} {} {})", left_code, op_s, right_code)
            }
            Expression::AddrOf(expr) => {
                format!("&({})", self.generate_expression(expr))
            }
            Expression::Deref(expr) => {
                format!("*({})", self.generate_expression(expr))
            }
            Expression::ConstChar(string) => {
                format!("nv_string_from_cstring(\"{}\")", string)
            }
            Expression::Cast {
                target_type: _,
                expr,
            } => self.generate_expression(expr),
            Expression::ConstChar(string) => string.clone(),
        }
    }

    fn generate_type(&self, ty: &NType) -> String {
        match ty {
            NType::Int => "int".to_string(),
            NType::Char => "char".to_string(),
            NType::I64 => "long".to_string(),
            NType::U8 => "unsigned char".to_string(),
            NType::String => "nv_string".to_string(),

            NType::Ptr(inner) => format!("{}*", self.generate_type(inner)),

            _ => "void*".to_string(),
        }
    }

    pub fn generate(&mut self) {
        let mut output = String::new();

        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <stdlib.h>\n");
        output.push_str("#include \"runtime/novac_rtm.h\"\n\n");

        output.push_str("int main(void) {\n");

        output.push_str(&format!(
            "    /* Novac Never-Value CodeGen v.{}{} */\n",
            self.version.as_str(),
            self.rep.as_str()
        ));

        let mut needs_to_return = true;

        for stmt in &self.program.statements {
            match stmt {
                Statement::Exit(expr) => {
                    needs_to_return = false;
                    match expr {
                        Expression::Int(val) => output.push_str(&format!("    exit({});\n", val)),
                        Expression::Var(var_name) => {
                            output.push_str(&format!("    exit({});\n", var_name))
                        }
                        Expression::BinaryOp { left, op, right } => {
                            output.push_str("    exit(");
                            output.push_str(&self.generate_expression(left));
                            output.push_str(");\n");
                        }
                        _ => {
                            todo!("BinOp");
                        }
                    }
                }
                Statement::Set { name, ty, val } => {
                    if let Some(value_expr) = val {
                        let expr_code = self.generate_expression(value_expr);
                        let c_type = self.generate_type(ty);
                        output.push_str(&format!("    {} {} = {};\n", c_type, name, expr_code));
                    }
                }

                Statement::PutChar(ascii_char) => {
                    let char_code = self.generate_expression(ascii_char);
                    output.push_str(&format!("    putchar({});\n", char_code));
                }

                Statement::If { condition, then_br, else_br } => {
                    output.push_str("    if ");
                    let expr = self.generate_expression(condition);
                    output.push_str(" {\n");
                    output.push_str("}\n");
                }

                Statement::CRtmPrint(s) => {
                    let final_string = self.generate_expression(s);
                    output.push_str(&format!("    nv_print_string(&{});\n", final_string));
                }

                _ => {
                    output.push_str("    // Unsupported statement\n");
                    warning!("Unsupported statement");
                    self.warnings += 1;
                }
            }
        }
        if needs_to_return {
            output.push_str("    return 0;\n");
        } else {
            output.push_str("   // No return needed, exit statement is present\n");
        }
        output.push_str("}\n");

        self.generated.push_str(&output);
    }

    pub fn to_c_file(&self, name: &str) {
        fs::write(name, &self.generated);
    }

    pub fn compile(
        &self,
        path: &str,
        rtm_lib_path: &str,
        rtm_include_path: &str,
    ) -> Result<(), String> {
        let read = fs::read_to_string(path).map_err(|e| format!("Failed to open file: {}", e))?;

        if read.is_empty() {
            return Err("File is empty".into());
        }

        println!("[*] Compiling {} to C...", { path });

        let status = Command::new("gcc")
            .args([
                path,
                "-o",
                "nnv_app",
                "-I",
                rtm_include_path,
                "-L",
                rtm_lib_path,
                "-l",
                "novac_rtm",
                "-O2",
            ])
            .status()
            .map_err(|e| format!("Failed to run GCC: {}", e))?;

        if status.success() {
            println!(
                "Compiled successfully with {} {}!",
                self.warnings,
                if self.warnings == 1 {
                    "warning"
                } else {
                    "warnings"
                }
            );
            Ok(())
        } else {
            Err("Compilation failed".into())
        }
    }
}
