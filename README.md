# Harissa CLI

Harissa is a command-line tool for managing background processes. It allows you to start applications in the background, monitor their status, and control their lifecycle with simple commands.

## Installation

Clone the repository and build the project with Cargo:

```bash
git clone https://github.com/yourusername/harissa.git
cd harissa
cargo build --release
```

The executable will be available at `target/release/harissa`.

## Commands

### help

Display help information about available commands.

```bash
harissa help [command]
```

If no command is specified, it lists all available commands. If a command is specified, it shows detailed help for that command.

### start

Start a command in the background and track it.

```bash
harissa start <command> [args...] [-n name]
```

Options:

- `-n <name>`: Specify a name for the background process (optional)
  - If not provided, the command name will be used
  - For multi-word names, provide them after the `-n` flag (e.g., `-n My App`)

Examples:

```bash
harissa start node app.js -n "Express Server"
harissa start python server.py
```

### list

List all running applications started with this CLI.

```bash
harissa list
```

For each application, displays:

- PID
- Name
- CPU usage (%)
- Memory usage (%)
- Command that was used to start it

### kill

Terminate a running application by PID or name.

```bash
harissa kill <PID | app name>
```

You can specify either the PID or the name of the application.

Examples:

```bash
harissa kill 1234
harissa kill "Express Server"
```

### reload

Restart running applications by PID or name.

```bash
harissa reload [PID | app name]
```

If no argument is provided, all running applications will be reloaded. If an argument is provided, only the specified application will be reloaded.

Examples:

```bash
harissa reload         # Reload all applications
harissa reload 1234    # Reload application with PID 1234
harissa reload "Express Server"  # Reload application named "Express Server"
```

### run

Execute a command and wait for it to complete.

```bash
harissa run <command> [args...]
```

Executes the specified command with the given arguments. The command runs in the foreground and the CLI waits for it to complete.

Example:

```bash
harissa run npm test
```

### startup

Generate a startup script for auto-starting applications on system boot.

```bash
harissa startup
```

Generates a shell script that can be used to automatically start all tracked applications when the system boots up. The script is saved to your home directory and installation commands for different Linux distributions are provided.

## How It Works

Harissa tracks all processes in `~/.harissa_apps/processes.csv` and maintains log files for each application in the same directory.

- Standard output is logged to `~/.harissa_apps/<app_name>.out.log`
- Standard error is logged to `~/.harissa_apps/<app_name>.err.log`

## Examples

### Starting a Node.js server with a custom name

```bash
harissa start node server.js -n "My API Server"
```

### Listing all running applications

```bash
harissa list
```

### Restarting a specific application

```bash
harissa reload "My API Server"
```

### Setting up system-wide autostart

```bash
harissa startup
# Follow the provided instructions for your Linux distribution
```

## License

[MIT License](LICENSE)
