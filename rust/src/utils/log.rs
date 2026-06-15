#[macro_export]
macro_rules! log {
    ($($args:tt)*) => {
        godot_print!("[{}] {}", module_path!(), format_args!($($args)*));
    };
}

#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        godot_error!(
            "[{}] {}. trace:\n{}",
            module_path!(),
            format_args!($($args)*),
            std::backtrace::Backtrace::force_capture(),
        );
    };
}
