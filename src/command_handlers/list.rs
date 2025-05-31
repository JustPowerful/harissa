use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::env;
use std::process::Command;

const HARISSA_APPS_DIR: &str = ".harissa_apps";

/// List all running applications started with this CLI
pub fn list_command(_args: &[String]) -> Result<(), String> {
    // Get the harissa directory
    let home_dir = env::var("HOME").map_err(|e| format!("Could not get HOME directory: {}", e))?;
    let harissa_dir = Path::new(&home_dir).join(HARISSA_APPS_DIR);
    let processes_file = harissa_dir.join("processes.csv");
    
    if !processes_file.exists() {
        println!("No applications are currently running.");
        return Ok(());
    }
    
    // Read the processes file
    let file = File::open(processes_file)
        .map_err(|e| format!("Failed to open processes file: {}", e))?;
    let reader = io::BufReader::new(file);
    
    // Prepare the table headers
    println!("{:<10} {:<20} {:<15} {:<15} {:<30}", "PID", "NAME", "CPU (%)", "MEMORY (%)", "COMMAND");
    println!("{:-<90}", "");
    
    // Track if we found any running processes
    let mut found_running = false;
    
    // Process each line
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() < 3 {
            continue;
        }
        
        let pid = parts[0];
        let name = parts[1];
        let command = parts[2..].join(",");
        
        // Check if the process is still running
        let check_process = Command::new("ps")
            .args(["-p", pid, "-o", "pid="])
            .output()
            .map_err(|e| format!("Failed to execute ps command: {}", e))?;
        
        if !check_process.status.success() {
            continue;
        }
        
        found_running = true;
        
        // Get CPU and memory usage using ps
        let usage = Command::new("ps")
            .args(["-p", pid, "-o", "pcpu,pmem"])  // Use pcpu,pmem format to get plain numbers without %
            .output()
            .map_err(|e| format!("Failed to get process usage: {}", e))?;
        
        let usage_str = String::from_utf8_lossy(&usage.stdout);
        let usage_parts: Vec<&str> = usage_str.trim().split_whitespace().collect();
        
        // Skip the header row if present (pcpu, pmem)
        let start_idx = if usage_parts.len() >= 2 && 
                          (usage_parts[0].contains("CPU") || usage_parts[0].contains("pcpu")) { 2 } else { 0 };
        
        let cpu = if usage_parts.len() > start_idx { format!("{}%", usage_parts[start_idx]) } else { "0.0%".to_string() };
        let mem = if usage_parts.len() > start_idx + 1 { format!("{}%", usage_parts[start_idx + 1]) } else { "0.0%".to_string() };
        
        println!("{:<10} {:<20} {:<15} {:<15} {:<30}", pid, name, cpu, mem, command);
    }
    
    if !found_running {
        println!("No applications are currently running.");
    }
    
    Ok(())
}
