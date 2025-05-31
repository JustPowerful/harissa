use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::env;
use std::process::{Command, Stdio};

const HARISSA_APPS_DIR: &str = ".harissa_apps";

/// Reload running applications
pub fn reload_command(args: &[String]) -> Result<(), String> {
    // Get the harissa directory
    let home_dir = env::var("HOME").map_err(|e| format!("Could not get HOME directory: {}", e))?;
    let harissa_dir = Path::new(&home_dir).join(HARISSA_APPS_DIR);
    let processes_file = harissa_dir.join("processes.csv");
    
    if !processes_file.exists() {
        return Err("No applications are currently running.".to_string());
    }

    // Read the processes file
    let file = File::open(&processes_file)
        .map_err(|e| format!("Failed to open processes file: {}", e))?;
    
    let reader = io::BufReader::new(file);
    let mut processes = Vec::new();
    let mut reloaded_count = 0;
    
    // Determine if we're reloading all apps or specific ones
    let reload_all = args.is_empty();
    let identifier = if !reload_all { args.join(" ") } else { String::new() };
    let is_pid = !reload_all && identifier.chars().all(|c| c.is_digit(10));
    
    // Process each line
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() < 3 {
            // Skip invalid lines
            processes.push(line);
            continue;
        }
        
        let pid = parts[0];
        let name = parts[1];
        let combined_command = parts[2..].join(",");
        let command_parts: Vec<&str> = combined_command.split_whitespace().collect();
        
        if command_parts.is_empty() {
            processes.push(line);
            continue;
        }
        
        // Check if this process matches our criteria for reloading
        let should_reload = reload_all || 
                           (is_pid && pid == identifier) || 
                           (!is_pid && name == identifier);
        
        if should_reload {
            // Check if the process is still running
            let check_process = Command::new("ps")
                .args(["-p", pid, "-o", "pid="])
                .output()
                .map_err(|e| format!("Failed to execute ps command: {}", e))?;
                
            if check_process.status.success() {
                // Process is running, kill it
                println!("Stopping process {} ({})...", name, pid);
                
                let kill_output = Command::new("kill")
                    .args([pid])
                    .output()
                    .map_err(|e| format!("Failed to execute kill command: {}", e))?;
                    
                if !kill_output.status.success() {
                    let error = String::from_utf8_lossy(&kill_output.stderr);
                    println!("Warning: Failed to kill process {} ({}): {}", name, pid, error);
                }
                
                // Give it a moment to shut down
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            
            // Start the process again
            println!("Restarting {}...", name);
            
            // Extract the command and its arguments
            let command = command_parts[0];
            let args = &command_parts[1..];
            
            // Prepare log files
            let stdout_log = harissa_dir.join(format!("{}.out.log", name));
            let stderr_log = harissa_dir.join(format!("{}.err.log", name));
            
            let stdout_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&stdout_log)
                .map_err(|e| format!("Failed to open stdout log file: {}", e))?;
                
            let stderr_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&stderr_log)
                .map_err(|e| format!("Failed to open stderr log file: {}", e))?;
            
            // Start the process
            let child = Command::new(command)
                .args(args)
                .stdout(Stdio::from(stdout_file))
                .stderr(Stdio::from(stderr_file))
                .spawn()
                .map_err(|e| format!("Failed to restart command: {}", e))?;
            
            // Get the new PID
            let new_pid = child.id();
            
            // Detach the child process
            std::mem::forget(child);
            
            // Create a new process entry with the new PID
            let new_process_info = format!("{},{},{}", new_pid, name, parts[2..].join(","));
            processes.push(new_process_info);
            
            println!("Restarted {} with new PID {}", name, new_pid);
            reloaded_count += 1;
        } else {
            // Keep this process entry unchanged
            processes.push(line);
        }
    }
    
    // Write the updated processes back to the file
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(processes_file)
        .map_err(|e| format!("Failed to open processes file for writing: {}", e))?;
        
    for process in processes {
        writeln!(file, "{}", process)
            .map_err(|e| format!("Failed to write to processes file: {}", e))?;
    }
    
    if reloaded_count == 0 {
        if reload_all {
            return Err("No running applications found to reload.".to_string());
        } else {
            return Err(format!("No running application found with {} '{}'", 
                if is_pid { "PID" } else { "name" }, identifier));
        }
    }
    
    println!("Successfully reloaded {} application(s).", reloaded_count);
    Ok(())
}
