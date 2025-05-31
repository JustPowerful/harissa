use std::collections::HashMap;

// Import command handlers
use crate::command_handlers::help_command;
use crate::command_handlers::run_command;
use crate::command_handlers::start_command;
use crate::command_handlers::list_command;
use crate::command_handlers::kill_command;
use crate::command_handlers::reload_command;
use crate::command_handlers::startup_command;

pub type CommandFn = fn(&[String]) -> Result<(), String>;

pub struct CommandRegistry {
    commands: HashMap<String, CommandFn>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut commands: HashMap<String, CommandFn> = HashMap::new();
        
        // Register all commands here
        commands.insert("help".to_string(), help_command);
        commands.insert("run".to_string(), run_command);
        commands.insert("start".to_string(), start_command);
        commands.insert("list".to_string(), list_command);
        commands.insert("kill".to_string(), kill_command);
        commands.insert("reload".to_string(), reload_command);
        commands.insert("startup".to_string(), startup_command);
        
        CommandRegistry { commands }
    }
    
    pub fn execute(&self, args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            return help_command(&[]);
        }
        
        let command_name = &args[0];
        let command_args = &args[1..];
        
        match self.commands.get(command_name) {
            Some(command_fn) => command_fn(command_args),
            None => Err(format!("Unknown command: {}", command_name)),
        }
    }
    
    pub fn get_command_names(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }
}


