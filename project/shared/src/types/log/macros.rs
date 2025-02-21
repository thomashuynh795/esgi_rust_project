// Makes the macro available to other modules.
#[macro_export]
// macro_rules! is the key work to declare a macro.
macro_rules! log_info {
    // The $()* syntax is used to match any number of arguments.
    // $arg is a variable that will hold the arguments passed to the macro.
    // tt is a token tree. It is a type of macro fragment.
    ($($arg:tt)*) => {
        // The macro is safe cause it uses the format! macro to format the message.
        $crate::types::log::log::Log::info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        $crate::types::log::log::Log::warning(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::types::log::log::Log::error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::types::log::log::Log::debug(&format!($($arg)*))
    };
}
