pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Self {}
    }

    pub fn info(&self, message: &str) {
        println!("from logger: {}", message);
    }

    pub fn error(&self, message: &str) {
        println!("from logger: {}", message);
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}
