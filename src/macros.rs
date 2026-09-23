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

#[macro_export]
macro_rules! warning {
    () => {
        eprintln!("\x1b[1;33m[Warning]\x1b[0m");
    };

    ($($arg:tt)*) => {
        eprintln!(
            "\x1b[1;33m[CodeGen warning]:\x1b[0m {}",
            format_args!($($arg)*)
        );
    }
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

pub fn run_with_spinner<F, T>(message: &'static str, task: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
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

            print!("\r\x1B[K{} {} ({:.2}s)", frame, message, elapsed);
            io::stdout().flush().unwrap();

            thread::sleep(Duration::from_millis(80));
            i += 1;
        }

        let total_elapsed = start_time.elapsed().as_secs_f64();

        print!("\r\x1B[K✔ {} ({:.2}s)\n", message, total_elapsed);
        print!("\x1B[?25h");
        io::stdout().flush().unwrap();
    });

    let task_handle = thread::spawn(task);
    let result = task_handle.join().unwrap();

    is_running.store(false, Ordering::Relaxed);
    spinner_handle.join().unwrap();

    result
}
