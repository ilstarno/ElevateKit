# ElevateKit

ElevateKit is a powerful Windows privilege elevation toolkit written in Rust that allows running processes with TrustedInstaller privileges. It provides a safe and efficient way to perform administrative tasks that require the highest level of system access.

## Features

- Elevate processes to TrustedInstaller privileges
- Safe Windows handle management using RAII patterns
- Proper process attribute handling
- Automatic service management
- Command-line argument support

## Installation

### Prerequisites

- Windows operating system
- Rust toolchain (1.75.0 or later recommended)
- Administrative privileges

### Building from Source

```bash
git clone https://github.com/ilstarno/ElevateKit.git
cd ElevateKit
cargo build --release
```

The compiled binary will be available at `target/release/elevatekit.exe`.

## Usage

ElevateKit can be run from an Administrator command prompt:

```bash
# Run PowerShell with TrustedInstaller privileges (default)
elevatekit

# Run specific command
elevatekit cmd.exe /c whoami

# Run PowerShell command
elevatekit powershell.exe -NoProfile -Command "Get-Process"
```

### Examples

1. List system files:
```bash
elevatekit cmd.exe /c dir "C:\Windows\System32"
```

2. Modify protected files:
```bash
elevatekit powershell.exe -Command "Set-Content -Path 'C:\Windows\System32\test.txt' -Value 'Hello'"
```

3. Check current privileges:
```bash
elevatekit powershell.exe -Command "whoami /priv"
```

## Security Considerations

- Always run ElevateKit from an elevated (Administrator) command prompt
- Be cautious when running commands with TrustedInstaller privileges
- Verify commands before execution to prevent system damage
- Do not use for malicious purposes

## Technical Details

ElevateKit works by:
1. Starting the TrustedInstaller service if not running
2. Creating a new process with TrustedInstaller as the parent
3. Setting appropriate process attributes and privileges
4. Managing Windows handles safely using Rust's RAII

## Development

### Project Structure
```
elevatekit/
├── src/
│   ├── core/
│   │   ├── elevation/      # Elevation strategies
│   │   └── process/        # Process management
│   ├── utils/
│   │   ├── security.rs     # Security context handling
│   │   └── error.rs        # Error types
│   └── main.rs             # Entry point
└── Cargo.toml
```

### Building with Different Features

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release
```

## Error Handling

ElevateKit provides detailed error messages for common issues:
- Service start failures
- Process creation errors
- Privilege elevation problems
- Handle management issues

## Contributing

Contributions are welcome! Please feel free to submit pull requests. Make sure to:

1. Follow the existing code style
2. Add tests for new functionality
3. Update documentation as needed
4. Ensure all tests pass

## License

MIT License - See [LICENSE](LICENSE) for details

## Acknowledgments

- Inspired by various Windows privilege elevation techniques
- Built using the Rust Windows API bindings

## Disclaimer

This tool is for legitimate system administration purposes only. Users are responsible for compliance with applicable laws and regulations.

## Author

Indrit Zeqiri - [Email](mailto:indrit.zeqiri@gmail.com)