# User Guide: Running & Developing CC-Switch

This guide covers how to run and develop the CC-Switch application locally.

## Prerequisites

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| [Node.js](https://nodejs.org/) | 20+ | Frontend runtime |
| [pnpm](https://pnpm.io/) | 10.10.0+ | Package manager |
| [Rust](https://www.rust-lang.org/) | 1.85+ | Backend runtime |
| [Cargo](https://doc.rust-lang.org/cargo/) | - | Rust package manager (included with Rust) |

### Checking Your Environment

```bash
# Check Node.js version
node --version  # Should be v20+

# Check pnpm version
pnpm --version   # Should be 10.10.0+

# Check Rust version
rustc --version   # Should be 1.85+

# Check Cargo version
cargo --version   # Should match Rust version
```

## Installation

### 1. Install Rust (if not already installed)

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment (or restart your terminal)
source $HOME/.cargo/env

# Verify installation
cargo --version
```

### 2. Install Dependencies

```bash
# Navigate to project directory
cd /path/to/cc-switch

# Install frontend dependencies
pnpm install
```

## Running the Application

### Development Mode

```bash
# Run the full development server (frontend + backend)
pnpm dev
```

This will start:
- **Frontend**: Vite dev server at `http://localhost:3000`
- **Backend**: Tauri Rust backend (compiled and launched automatically)

The application window should open automatically.

### Individual Components

```bash
# Run only the frontend dev server
pnpm dev:renderer

# Run only the backend (requires frontend to be running)
cd src-tauri
cargo run
```

## Common Issues & Solutions

### Issue: "cargo: command not found"

**Cause**: Cargo is not in your PATH.

**Solution**:
```bash
# Add Cargo to PATH (temporary)
source $HOME/.cargo/env

# Or add to your shell profile (permanent)
echo 'source $HOME/.cargo/env' >> ~/.zshrc   # for zsh
echo 'source $HOME/.cargo/env' >> ~/.bashrc  # for bash
```

### Issue: "Port 3000 is already in use"

**Cause**: Another process is using port 3000.

**Solution**:
```bash
# Find and kill the process using port 3000
lsof -ti:3000 | xargs kill -9

# Then try running again
pnpm dev
```

### Issue: "failed to run 'cargo metadata'"

**Cause**: Cargo is not installed or not in PATH.

**Solution**: See the Rust installation section above.

### Issue: Rust compilation errors

**Cause**: Dependencies may not be up to date.

**Solution**:
```bash
# Clean and rebuild
cd src-tauri
cargo clean
cargo build
```

## Build for Production

```bash
# Build the release version
pnpm build

# The output will be in:
# - macOS: src-tauri/target/release/bundle/macos/
# - Windows: src-tauri/target/release/bundle/windows/
# - Linux: src-tauri/target/release/bundle/appimage/deb/
```

## Development Scripts

| Command | Description |
|---------|-------------|
| `pnpm dev` | Start development server (frontend + backend) |
| `pnpm dev:renderer` | Start only frontend dev server |
| `pnpm build` | Build production release |
| `pnpm build:renderer` | Build only frontend |
| `pnpm typecheck` | Run TypeScript type checking |
| `pnpm format` | Format code with Prettier |
| `pnpm format:check` | Check code formatting |
| `pnpm test:unit` | Run unit tests |
| `pnpm test:unit:watch` | Run unit tests in watch mode |

## Project Structure

```
cc-switch/
├── src/                    # Frontend source code
│   ├── components/         # React components
│   ├── hooks/              # Custom React hooks
│   ├── lib/                # Utility functions
│   └── ...                 # Other frontend files
├── src-tauri/              # Backend Rust code
│   ├── src/
│   │   ├── commands/       # Tauri commands
│   │   ├── database/       # Database layer
│   │   ├── services/       # Business logic
│   │   └── ...             # Other backend files
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── package.json            # Node.js dependencies
└── pnpm-lock.yaml          # Lock file for pnpm
```

## Troubleshooting

### Clean Build

If you encounter persistent issues:

```bash
# 1. Clean node modules
rm -rf node_modules
pnpm install

# 2. Clean Rust build cache
cd src-tauri
cargo clean

# 3. Rebuild and run
cd ..
pnpm dev
```

### Check Dependencies

```bash
# Check if all required tools are installed
node --version && pnpm --version && cargo --version && rustc --version
```

### View Build Logs

If the build fails, check the terminal output for detailed error messages. Rust compiler errors are typically displayed with:
- File path and line number
- Error type (e.g., `E0277`, `E0382`)
- Detailed description
- Suggested fixes

## Resources

- [Tauri Documentation](https://tauri.app/)
- [React Documentation](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [pnpm Documentation](https://pnpm.io/)
