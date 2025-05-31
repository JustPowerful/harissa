mod commands;
mod command_handlers;

use std::env;
use std::process;
use commands::CommandRegistry;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Skip the program name (args[0])
    let command_args = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        Vec::new()
    };
    
    let registry = CommandRegistry::new();
    
    match registry.execute(&command_args) {
        Ok(()) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
