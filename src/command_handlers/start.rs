use std::process::{Command, Stdio};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::env;

const HARISSA_APPS_DIR: &str = ".harissa_apps";

/// Start a command in the background and track it
pub fn start_command(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: start <command> [args...]\nExample: start node app.js -n my_app".to_string());
    }

    // Extract the name if provided with -n flag
    let mut app_name = None;
    let mut filtered_args = Vec::new();
    
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-n" {
            // Check if we have more arguments and collect all of them until the next flag or end
            let mut name_parts = Vec::new();
            i += 1; // Skip the -n flag
            
            // Collect all parts until we find another flag or reach the end
            while i < args.len() && !args[i].starts_with('-') {
                name_parts.push(args[i].clone());
                i += 1;
            }
            
            if !name_parts.is_empty() {
                app_name = Some(name_parts.join(" "));
            } else if i < args.len() {
                // If we immediately hit another flag or the end, use a default name
                app_name = Some(format!("app_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));
            }
        } else {
            filtered_args.push(args[i].clone());
            i += 1;
        }
    }

    // If no name was provided, use the command as the name
    let app_name = app_name.unwrap_or_else(|| filtered_args[0].clone());
    
    // Ensure the command exists
    if filtered_args.is_empty() {
        return Err("No command specified after removing flags".to_string());
    }

    // Create the command
    let command = &filtered_args[0];
    let args = &filtered_args[1..];

    // Ensure directory for tracking apps exists
    let home_dir = env::var("HOME").map_err(|e| format!("Could not get HOME directory: {}", e))?;
    let harissa_dir = Path::new(&home_dir).join(HARISSA_APPS_DIR);
    
    if !harissa_dir.exists() {
        std::fs::create_dir_all(&harissa_dir)
            .map_err(|e| format!("Failed to create directory for tracking apps: {}", e))?;
    }

    // Prepare log files
    let stdout_log = harissa_dir.join(format!("{}.out.log", app_name));
    let stderr_log = harissa_dir.join(format!("{}.err.log", app_name));
    
    let stdout_file = File::create(&stdout_log)
        .map_err(|e| format!("Failed to create stdout log file: {}", e))?;
    let stderr_file = File::create(&stderr_log)
        .map_err(|e| format!("Failed to create stderr log file: {}", e))?;

    // Start the process
    let child = Command::new(command)
        .args(args)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|e| format!("Failed to start command: {}", e))?;

    // Get the PID
    let pid = child.id();
    
    // Detach the child process - we don't need to wait for it
    std::mem::forget(child);
    
    // Save the process info
    let process_info = format!("{},{},{}", pid, app_name, filtered_args.join(" "));
    let processes_file = harissa_dir.join("processes.csv");
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(processes_file)
        .map_err(|e| format!("Failed to open processes file: {}", e))?;
        
    writeln!(file, "{}", process_info)
        .map_err(|e| format!("Failed to write to processes file: {}", e))?;

    println!("Started {} with PID {} in the background", app_name, pid);
    println!("Logs available at:");
    println!("  Stdout: {}", stdout_log.display());
    println!("  Stderr: {}", stderr_log.display());
    
    Ok(())
}
