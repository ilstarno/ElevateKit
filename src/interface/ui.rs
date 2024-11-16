use colored::*;

pub struct UI;

impl UI {
    pub fn print_banner() {
        println!("{}", r#"
╔════════════════════════════════════════════════════════════╗
║                   TrustedInstaller Tool                    ║
║                                                           ║
║  A powerful utility for elevated Windows operations       ║
╚════════════════════════════════════════════════════════════╝
        "#.bright_cyan());
    }

    pub fn print_status(operation: &str, status: bool) {
        let symbol = if status { "✓".green() } else { "✗".red() };
        println!(" {} {} {}", 
            symbol, 
            operation, 
            if status { "Success".green() } else { "Failed".red() }
        );
    }
} 