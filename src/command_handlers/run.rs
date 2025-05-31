/// Example of a run command
pub fn run_command(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("The run command requires at least one argument".to_string());
    }
    
    println!("Running with arguments: {:?}", args);
    
    // Implement your command logic here
    
    Ok(())
}
