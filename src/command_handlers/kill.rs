use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::env;
use std::process::Command;

const HARISSA_APPS_DIR: &str = ".harissa_apps";

/// Kill a running application by PID or name
pub fn kill_command(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: kill <PID | app name>\nExample: kill 1234\nExample: kill my_app".to_string());
    }

    // Check if the first argument is a PID (all digits)
    let is_pid = args[0].chars().all(|c| c.is_digit(10));
    
    // Use either the PID or join all arguments as the app name
    let identifier = if is_pid {
        args[0].clone()
    } else {
        args.join(" ")
    };
    
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
    let mut found_process = false;
    let mut killed_pids = Vec::new();
    
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
        
        // Check if this process matches our identifier
        let is_match = if is_pid {
            pid == identifier
        } else {
            name == identifier
        };
        
        if is_match {
            found_process = true;
            
            // Check if the process is still running
            let check_process = Command::new("ps")
                .args(["-p", pid, "-o", "pid="])
                .output()
                .map_err(|e| format!("Failed to execute ps command: {}", e))?;
                
            if check_process.status.success() {
                // Process is running, kill it
                let kill_output = Command::new("kill")
                    .args([pid])
                    .output()
                    .map_err(|e| format!("Failed to execute kill command: {}", e))?;
                    
                if kill_output.status.success() {
                    println!("Successfully terminated process {} ({})", name, pid);
                    killed_pids.push(pid.to_string());
                } else {
                    let error = String::from_utf8_lossy(&kill_output.stderr);
                    println!("Failed to kill process {} ({}): {}", name, pid, error);
                    processes.push(line);
                }
            } else {
                // Process is not running, remove it from the list
                println!("Process {} ({}) is not running", name, pid);
                killed_pids.push(pid.to_string());
            }
        } else {
            // Check if the process is still running before adding it back
            let check_process = Command::new("ps")
                .args(["-p", pid, "-o", "pid="])
                .output()
                .map_err(|e| format!("Failed to execute ps command: {}", e))?;
                
            if check_process.status.success() {
                // Process is still running, keep it in the list
                processes.push(line);
            } else {
                // Process is not running, don't add it back
                killed_pids.push(pid.to_string());
            }
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
    
    if !found_process {
        return Err(format!("No running application found with {} '{}'", 
            if is_pid { "PID" } else { "name" }, identifier));
    }
    
    // Also clean up any log files for killed processes
    for pid in killed_pids {
        // We don't have the name here, but the PID might be in the filename
        // This is just a best-effort cleanup
        let _ = std::fs::remove_file(harissa_dir.join(format!("{}.out.log", pid)));
        let _ = std::fs::remove_file(harissa_dir.join(format!("{}.err.log", pid)));
    }
    
    Ok(())
}
