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
