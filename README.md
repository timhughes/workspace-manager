# Workspace Manager for VSCode

A command-line tool to automatically generate VS Code workspace files from directory structures.

## Features

- 📁 Scans directories and creates workspace entries
- 🏗️ Includes root directory by default
- 🔄 Updates existing workspace files
- 🔧 Configurable workspace task
- 🚫 Ignores hidden folders

## Installation

### Install from source

Prerequisites: 

- Rust toolchain (1.70 or later)
- Git
- Cargo

```bash
# Clone the repository
git clone https://github.com/timhughes/workspace-manager.git
cd workspace-manager

# Build and install
cargo build --release
cargo install --path .
```

## Usage

```bash
# Create workspace file for current directory
workspace-manager

# Scan specific path (includes current directory by default)
workspace-manager --path /path/to/projects

# Exclude current directory
workspace-manager -p /path/to/projects --exclude-current

# Custom workspace name and update tasks
workspace-manager -p . -n my-workspace --update-task
```

## CLI Options

- `-p, --path <PATH>`: Directory to scan (default: current directory)
- `-e, --exclude-current`: Exclude current directory from workspace
- `-n, --name <NAME>`: Custom name for workspace file
- `-u, --update-tasks`: Force update of workspace tasks

## License

MIT - See [LICENSE](LICENSE) file for details
