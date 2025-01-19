# Workspace Manager

A command-line tool to automatically generate VS Code workspace files from directory structures.

## Features

- 📁 Scans directories and creates workspace entries
- 🏗️ Optional inclusion of root directory
- 🔄 Updates existing workspace files
- 🔧 Configurable workspace tasks
- 🚫 Ignores hidden folders

## Installation

### Building from source

Prerequisites:

- Rust toolchain (1.70 or later)
- Git

```bash
# Clone the repository
git clone https://github.com/timhughes/workspace-manager.git
cd workspace-manager

# Build the project
cargo build --release

# Install locally
cargo install --path .
```

## Usage

```bash
# Create workspace file for current directory
workspace-manager

# Scan specific path
workspace-manager --path /path/to/project

# Include current directory and custom name
workspace-manager -p /path/to/project -i -n my-workspace

# Update tasks in existing workspace
workspace-manager -p . --update-tasks
```

## CLI Options

- `-p, --path <PATH>`: Directory to scan (default: current directory)
- `-i, --include-current`: Add current directory as first workspace folder
- `-n, --name <NAME>`: Custom name for workspace file
- `-u, --update-tasks`: Force update of workspace tasks

## License

MIT - See [LICENSE](LICENSE) file for details
