# embedded-gdb-mcp

[![Crates.io](https://img.shields.io/crates/v/embedded-gdb-mcp.svg)](https://crates.io/crates/embedded-gdb-mcp)
[![license](https://img.shields.io/github/license/vividcode-ai/embedded-gdb-mcp.svg)](https://github.com/vividcode-ai/embedded-gdb-mcp/blob/main/LICENSE)

A Model Context Protocol (MCP) server that provides GDB debugging functionality, built with Rust for embedded development workflows.

## Features

- Start and manage GDB debugging sessions
- Load programs and core dumps for analysis
- Set breakpoints with conditional support
- Step through code (line by line or instruction by instruction)
- Examine memory, registers, and variables
- View call stacks and source code with VS Code integration
- Execute arbitrary GDB commands
- Attach to running processes
- Pre-built binaries for Windows, macOS, and Linux (x64 & arm64)

## Installation

### Claude Code

```bash
claude mcp add embedded-gdb-debugger -- npx -y @vividcodeai/embedded-gdb-mcp
```

### Claude Desktop

Add the following to your Claude Desktop MCP configuration:

```json
{
  "mcpServers": {
    "embedded-gdb-debugger": {
      "command": "npx",
      "args": ["-y", "@vividcodeai/embedded-gdb-mcp"]
    }
  }
}
```

### Install from Source

```bash
git clone https://github.com/vividcode-ai/embedded-gdb-mcp.git
cd embedded-gdb-mcp
cargo build --release
```

The binary will be available at `target/release/embedded-gdb-mcp`.

## Usage

### Example Commands

#### Starting a GDB session

```
Use gdb_start to start a new debugging session
```

#### Loading a program

```
Use gdb_load to load /path/to/my/program with the sessionId that was returned from gdb_start
```

#### Setting a breakpoint

```
Use gdb_set_breakpoint to set a breakpoint at main in the active GDB session
```

#### Running the program

```
Use gdb_continue to start execution
```

#### Examining variables

```
Use gdb_print to evaluate the expression "my_variable" in the current context
```

#### Getting a backtrace

```
Use gdb_backtrace to see the current call stack
```

#### Terminating the session

```
Use gdb_terminate to end the debugging session
```

## Supported GDB Commands

- `gdb_start`: Start a new GDB session
- `gdb_load`: Load a program into GDB
- `gdb_command`: Execute an arbitrary GDB command
- `gdb_terminate`: Terminate a GDB session
- `gdb_list_sessions`: List all active GDB sessions
- `gdb_attach`: Attach to a running process
- `gdb_load_core`: Load a core dump file
- `gdb_set_breakpoint`: Set a breakpoint (with optional condition)
- `gdb_continue`: Continue program execution
- `gdb_step`: Step program execution (line or instruction)
- `gdb_next`: Step over function calls (line or instruction)
- `gdb_finish`: Execute until the current function returns
- `gdb_backtrace`: Show call stack (with optional full mode and frame limit)
- `gdb_print`: Print value of expression
- `gdb_examine`: Examine memory (with format and count options)
- `gdb_info_registers`: Display registers (optionally for a specific register)
- `gdb_list_source`: List source code at current or specified location with VS Code integration

## License

MIT

---

[中文文档](README.zh.md)
