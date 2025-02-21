pub struct Log;

impl Log {
    pub fn error(message: &str) {
        println!("🔴 ERROR   {}", message);
    }

    pub fn warning(message: &str) {
        println!("🟠 WARNING {}", message);
    }

    pub fn info(message: &str) {
        println!("🔵 INFO    {}", message);
    }

    pub fn debug(message: &str) {
        println!("🟣 DEBUG   {}", message);
    }
}
