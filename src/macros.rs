use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crate::enums::{Expression, Op, Program, Statement};

use colored::*;

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        eprint!("{}: ", "[?] Warning".bold().yellow());
        eprintln!($($arg)*);
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        eprint!("{}: ", "[!] Error".bold().red());
        eprintln!($($arg)*);
    }};
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        println!("{}: ", "[*] Success".bold().green());
        println!($($arg)*);
    }};
}

fn format_op(op: &Op) -> String {
    match op {
        Op::Add => "+".to_string(),
        Op::Sub => "-".to_string(),
        Op::Mul => "*".to_string(),
        Op::Div => "/".to_string(),
        Op::GreaterEq => ">=".to_string(),
        Op::LessEq => "<=".to_string(),
        Op::Less => "<".to_string(),
        Op::Greater => ">".to_string(),
        Op::NotEq => "!=".to_string(),
        Op::Equal => "==".to_string(),
    }
}

fn format_value(expr: &Expression) -> String {
    match expr {
        Expression::Int(intv) => intv.to_string(),
        Expression::Var(varv) => varv.to_string(),
        Expression::Char(charv) => charv.to_string(),
        Expression::ConstChar(ccv) => ccv.to_string(),

        Expression::BinaryOp { left, op, right } => {
            format!(
                "{} {} {}",
                format_value(left),
                format_op(op),
                format_value(right)
            )
        }

        _ => "none".to_string(),
    }
}

pub fn show_parsed_nicely(prg: &Program) {
    let mut counter = 1;
    for stmt in &prg.statements {
        match stmt {
            Statement::Set { name, ty, val } => {
                let unw_val = format_value(val.as_ref().unwrap());

                println!("| {} : Set ", counter);
                println!("|     name: {}", name);
                println!("|     type: {:?}", ty);
                println!("|     val:  {}", unw_val);
                println!("-                    ");
            }
            Statement::Assign { name, val } => {
                let unw_val = format_value(val);

                println!("| {} : Assign ", counter);
                println!("|     to: {}", name);
                println!("|     from: {}", unw_val);
                println!("-                    ");
            }
            Statement::Exit(v) => {
                let unw_val = format_value(v);

                println!("| {} : Exit ", counter);
                println!("|     exit_code: {}", unw_val);
                println!("-                    ");
            }
            _ => {}
        }
        counter += 1;
    }
}

pub fn print_ast(prg: &Program) {
    println!("{}", "root [Program]".bold().cyan());
    let count = prg.statements.len();
    for (index, stmt) in prg.statements.iter().enumerate() {
        let is_last = index == count - 1;
        print_statement(stmt, "", is_last);
    }
}

fn print_statement(stmt: &Statement, pref: &str, is_last: bool) {
    let marker = if is_last { "└─" } else { "├─" };
    let ch_pref = format!("{}{}", pref, if is_last { "    " } else { "│   " });

    match stmt {
        Statement::Set { name, ty, val } => {
            println!("{}{}{}: {:?}", pref, marker, "Set".bold().yellow(), name);
            println!("{}├── type: {:?}", ch_pref, ty);
            if let Some(v) = val {
                print_expression(v, &ch_pref, true, "val");
            }
        }

        Statement::Assign { name, val } => {
            println!("{}{}{}: {}", pref, marker, "Assign".bold().blue(), name);
            print_expression(val, &ch_pref, true, "val");
        }

        Statement::Use(inner) => {
            println!("{}{}{}: Use", pref, marker, "Use".bold().magenta());
            print_statement(inner, &ch_pref, true);
        }

        Statement::Import(lib) => {
            println!("{}{}{}: {}", pref, marker, "Import".bold().cyan(), lib);
        }

        Statement::Module(name) => {
            println!("{}{}{}: {}", pref, marker, "Module".bold().cyan(), name);
        }

        Statement::Exit(v) => {
            println!("{}{}{}", pref, marker, "Exit".bold().red());
            print_expression(v, &ch_pref, true, "code");
        }

        _ => {
            println!("{}{}{:?}", pref, marker, stmt);
        }
    }
}

fn print_expression(expr: &Expression, pref: &str, is_last: bool, label: &str) {
    let marker = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", pref, if is_last { "    " } else { "│   " });

    match expr {
        Expression::BinaryOp { left, op, right } => {
            println!("{}{}{}: op({:?})", pref, marker, label.dimmed(), op);
            print_expression(left, &child_prefix, false, "left");
            print_expression(right, &child_prefix, true, "right");
        }
        Expression::Int(val) => {
            println!("{}{}{}: {} (i64)", pref, marker, label.dimmed(), val);
        }
        Expression::Var(name) => {
            println!("{}{}{}: var({})", pref, marker, label.dimmed(), name);
        }
        _ => {
            println!("{}{}{}: {:?}", pref, marker, label.dimmed(), expr);
        }
    }
}

pub fn run_with_spinner<F, T>(message: &'static str, task: F) -> T
where
    F: FnOnce() -> T,
{
    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_clone = Arc::clone(&is_running);

    let start_time = Instant::now();

    let spinner_handle = thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;
        print!("\x1B[?25l");

        while is_running_clone.load(Ordering::Relaxed) {
            let frame = frames[i % frames.len()];
            let elapsed = start_time.elapsed().as_secs_f64();

            print!(
                "\r\x1B[K{} {} ({}s)",
                frame.bold().green(),
                message.cyan(),
                elapsed.to_string().dimmed()
            );
            io::stdout().flush().unwrap();

            thread::sleep(Duration::from_millis(80));
            i += 1;
        }

        let total_elapsed = start_time.elapsed().as_secs_f64();

        print!(
            "\r\x1B[K{} {} ({}s)\n",
            "✔".bold().green(),
            message.cyan(),
            total_elapsed.to_string().dimmed()
        );
        print!("\x1B[?25h");
        io::stdout().flush().unwrap();
    });

    let result = task();

    is_running.store(false, Ordering::Relaxed);
    spinner_handle.join().unwrap();

    result
}
