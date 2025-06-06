use crate::commands::CommandRegistry;

/// Displays help information about available commands
pub fn help_command(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        println!("Available commands:");
        let registry = CommandRegistry::new();
        for cmd in registry.get_command_names() {
            println!("  {}", cmd);
        }
        println!("\nUse 'help <command>' for more information about a specific command.");
    } else {
        let command = &args[0];
        match command.as_str() {
            "help" => {
                println!("help - Display help information about available commands");
                println!("\nUsage: help [command]");
                println!("  If no command is specified, lists all available commands.");
                println!("  If a command is specified, shows detailed help for that command.");
            },
            "run" => {
                println!("run - Execute a command and wait for it to complete");
                println!("\nUsage: run <command> [args...]");
                println!("  Executes the specified command with the given arguments.");
                println!("  The command runs in the foreground and the CLI waits for it to complete.");
            },
            "start" => {
                println!("start - Start a command in the background");
                println!("\nUsage: start <command> [args...] [-n name]");
                println!("  Starts the specified command in the background and tracks it.");
                println!("  Options:");
                println!("    -n <name>  Specify a name for the background process (optional)");
                println!("               If not provided, the command name will be used");
                println!("\nExample: start node app.js -n my_app");
                println!("Example: start python server.py");
            },
            "list" => {
                println!("list - List all running applications started with this CLI");
                println!("\nUsage: list");
                println!("  Shows information about all currently running applications that were");
                println!("  started using the 'start' command.");
                println!("  For each application, displays the PID, name, CPU usage, memory usage,");
                println!("  and the command that was used to start it.");
            },
            "kill" => {
                println!("kill - Terminate a running application by PID or name");
                println!("\nUsage: kill <PID | app name>");
                println!("  Terminates a running application that was started with the 'start' command.");
                println!("  You can specify either the PID or the name of the application.");
                println!("\nExample: kill 1234");
                println!("Example: kill my_app");
            },
            "reload" => {
                println!("reload - Restart running applications by PID or name");
                println!("\nUsage: reload [PID | app name]");
                println!("  Restarts running applications that were started with the 'start' command.");
                println!("  If no argument is provided, all running applications will be reloaded.");
                println!("  If an argument is provided, only the specified application will be reloaded.");
                println!("\nExample: reload         # Reload all applications");
                println!("Example: reload 1234    # Reload application with PID 1234");
                println!("Example: reload my_app  # Reload application named my_app");
            },
            "startup" => {
                println!("startup - Generate a startup script for auto-starting applications on system boot");
                println!("\nUsage: startup");
                println!("  Generates a shell script that can be used to automatically start all tracked");
                println!("  applications when the system boots up.");
                println!("  The script is printed to the console and can be redirected to a file.");
                println!("  Instructions for setting up the startup script are provided when run.");
                println!("\nExample: startup > ~/harissa_startup.sh");
            },
            _ => {
                println!("Help for command: {}", command);
                println!("No detailed help available for this command.");
            }
        }
    }
    Ok(())
}
