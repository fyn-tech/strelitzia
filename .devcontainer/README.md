# Rust Development Container

This devcontainer provides a complete Rust development environment with all necessary tools and VS Code extensions.

## What's Included

### Rust Toolchain
- Latest stable Rust compiler
- Cargo package manager
- Clippy linter
- Rustfmt code formatter
- Rust source code for better IDE support

### VS Code Extensions
- **rust-analyzer**: Advanced Rust language server
- **CodeLLDB**: Debugging support
- **crates**: Crate dependency management
- **Even Better TOML**: Enhanced TOML file support

### Additional Tools
- **Built-in Cargo commands**: Modern dependency management (`cargo add`, `cargo remove`)
- **Git**: Version control
- **GitHub CLI**: GitHub integration

## Getting Started

1. **Prerequisites**: Install Docker and VS Code with the Dev Containers extension
2. **Open in Container**: 
   - Open this folder in VS Code
   - When prompted, click "Reopen in Container"
   - Or use Command Palette: "Dev Containers: Reopen in Container"

3. **Create a new Rust project**:
   ```bash
   cargo new my_project
   cd my_project
   cargo run
   ```

## Features

- **Auto-formatting**: Code is automatically formatted on save
- **Linting**: Clippy runs on save to catch common mistakes
- **IntelliSense**: Full code completion and navigation
- **Debugging**: Integrated debugging with breakpoints
- **Port Forwarding**: Ports 3000, 8000, and 8080 are automatically forwarded

## Customization

You can modify the devcontainer configuration by editing:
- `.devcontainer/devcontainer.json`: VS Code settings and extensions
- `.devcontainer/Dockerfile`: Container image and system packages
