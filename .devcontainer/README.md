# Rust Development Container

This devcontainer provides a complete Rust development environment with all necessary tools and VS Code extensions.

## What's Included

### Rust Toolchain
- Rust 1.91.0 compiler with Cargo package manager (includes built-in `cargo add`/`cargo remove`)
- Clippy linter (runs on save to catch common mistakes)
- Rustfmt code formatter (auto-formats on save)
- Rust source code for better IDE support

### VS Code Extensions
- **rust-analyzer**: Advanced language server with IntelliSense, code completion, and navigation
- **CodeLLDB**: Integrated debugging with breakpoints
- **Dependi**: Crate dependency management and version updates
- **Even Better TOML**: Enhanced TOML file support
- **GitHub Actions**: Workflow syntax highlighting and validation

### Additional Tools
- **Git**: Version control
- **Python 3**: For matplotlib-based visualization

## Getting Started

1. **Prerequisites**: Install [Docker](https://www.docker.com/) and VS Code with the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

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

## Testing

Run the environment verification script to check that all tools are correctly installed:

```bash
./.devcontainer/test-setup.sh
```

This script is also run automatically in CI when changes are made to the `.devcontainer/` directory.

## Customization

You can modify the devcontainer configuration by editing:
- `.devcontainer/devcontainer.json`: VS Code settings and extensions
- `.devcontainer/Dockerfile`: Container image and system packages

**Note**: Ports 3000, 8000, and 8080 are configured for automatic forwarding for web development.
