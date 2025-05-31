use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::env;
use std::process::Command;

const HARISSA_APPS_DIR: &str = ".harissa_apps";

/// Detect the Linux distribution
fn detect_linux_distribution() -> Result<String, String> {
    // Check if /etc/os-release exists
    if Path::new("/etc/os-release").exists() {
        let output = Command::new("sh")
            .args(["-c", "cat /etc/os-release | grep ^ID= | cut -d= -f2"])
            .output()
            .map_err(|e| format!("Failed to detect distribution: {}", e))?;
            
        let distro = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Remove quotes if present
        let distro = distro.trim_matches('"').trim_matches('\'').to_string();
        
        if !distro.is_empty() {
            return Ok(distro);
        }
    }
    
    // Try lsb_release as fallback
    let output = Command::new("lsb_release")
        .args(["-is"])
        .output();
        
    if let Ok(output) = output {
        let distro = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        if !distro.is_empty() {
            return Ok(distro);
        }
    }
    
    // Default fallback
    Ok("linux".to_string())
}

/// Generate a startup script and system service setup command to auto-start applications on system boot
pub fn startup_command(args: &[String]) -> Result<(), String> {
    // Check if we need to determine the distribution
    let detect_distro = args.is_empty() || args[0] != "--skip-detect";
    // Get the current user for systemd user services
    let _user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let home_dir = env::var("HOME").map_err(|e| format!("Could not get HOME directory: {}", e))?;
    
    // Detect Linux distribution if needed
    let distribution = if detect_distro {
        detect_linux_distribution()?
    } else {
        "linux".to_string()
    };
    // Get the harissa directory
    let harissa_dir = Path::new(&home_dir).join(HARISSA_APPS_DIR);
    let processes_file = harissa_dir.join("processes.csv");
    
    if !processes_file.exists() {
        return Err("No applications are currently running. Nothing to add to startup.".to_string());
    }

    // Read the processes file
    let file = File::open(&processes_file)
        .map_err(|e| format!("Failed to open processes file: {}", e))?;
    
    let reader = io::BufReader::new(file);
    let mut script_contents = String::new();
    
    // Add shebang and header
    script_contents.push_str("#!/bin/bash\n\n");
    script_contents.push_str("# Auto-generated startup script for harissa applications\n");
    script_contents.push_str("# Generated on: ");
    script_contents.push_str(&chrono::Local::now().to_string());
    script_contents.push_str("\n\n");
    
    let harissa_path = env::current_exe()
        .map_err(|e| format!("Failed to get harissa executable path: {}", e))?;
    
    script_contents.push_str("# Path to harissa executable\n");
    script_contents.push_str(&format!("HARISSA=\"{}\"\n\n", harissa_path.display()));
    
    let mut app_count = 0;
    
    // Process each line
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() < 3 {
            // Skip invalid lines
            continue;
        }
        
        let name = parts[1];
        let command = parts[2..].join(",");
        
        script_contents.push_str(&format!("# Start {}\n", name));
        script_contents.push_str(&format!("$HARISSA start -n \"{}\" {}\n\n", name, command));
        
        app_count += 1;
    }
    
    if app_count == 0 {
        return Err("No valid applications found to add to startup.".to_string());
    }

    // Create startup script path
    let startup_script_path = Path::new(&home_dir).join("harissa_startup.sh");
    
    // Write the script to a file
    let mut script_file = File::create(&startup_script_path)
        .map_err(|e| format!("Failed to create startup script: {}", e))?;
    
    write!(script_file, "{}", script_contents)
        .map_err(|e| format!("Failed to write to startup script: {}", e))?;
    
    // Make the script executable
    Command::new("chmod")
        .args(["+x", startup_script_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("Failed to make startup script executable: {}", e))?;
    
    // Print out the script
    println!("\nStartup script saved to: {}", startup_script_path.display());
    println!("\nScript contents:");
    println!("{}", script_contents);
    
    // Generate the system service installation command
    println!("\nTo install as a system service, run the following command:");
    
    let path_var = env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".to_string());
    
    match distribution.as_str() {
        "ubuntu" | "debian" | "linuxmint" => {
            println!("sudo su -c \"env PATH={} {} startup --skip-detect > /etc/init.d/harissa-startup && chmod +x /etc/init.d/harissa-startup && update-rc.d harissa-startup defaults\" -s /bin/sh",
                     path_var, harissa_path.display());
        },
        "fedora" | "rhel" | "centos" | "rocky" | "almalinux" => {
            println!("sudo su -c \"env PATH={} {} startup --skip-detect > /etc/systemd/system/harissa-startup.service && systemctl enable harissa-startup.service\" -s /bin/sh",
                     path_var, harissa_path.display());
        },
        "arch" | "manjaro" => {
            println!("sudo su -c \"env PATH={} {} startup --skip-detect > /etc/systemd/system/harissa-startup.service && systemctl enable harissa-startup.service\" -s /bin/sh",
                     path_var, harissa_path.display());
        },
        _ => {
            // Generic systemd (most Linux distributions use systemd nowadays)
            println!("sudo su -c \"env PATH={} {} startup --skip-detect > /etc/systemd/system/harissa-startup.service && systemctl enable harissa-startup.service\" -s /bin/sh",
                     path_var, harissa_path.display());
            println!("\nIf your system doesn't use systemd, you can add the startup script to /etc/rc.local or the appropriate startup mechanism for your distribution.");
        }
    }
    
    // Instructions for user-level startup (without sudo)
    println!("\nFor user-level startup (without sudo):");
    println!("mkdir -p ~/.config/systemd/user/");
    println!("cat << EOF > ~/.config/systemd/user/harissa.service");
    println!("[Unit]");
    println!("Description=Harissa Applications Startup");
    println!("After=network.target");
    println!("");
    println!("[Service]");
    println!("Type=oneshot");
    println!("ExecStart=/bin/bash {}", startup_script_path.display());
    println!("");
    println!("[Install]");
    println!("WantedBy=default.target");
    println!("EOF");
    println!("systemctl --user enable harissa.service");
    
    println!("\nOr for crontab:");
    println!("crontab -e");
    println!("Then add: @reboot {}", startup_script_path.display());
    
    Ok(())
}
