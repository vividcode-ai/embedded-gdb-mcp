# embedded-gdb-mcp

[![Crates.io](https://img.shields.io/crates/v/embedded-gdb-mcp.svg)](https://crates.io/crates/embedded-gdb-mcp)
[![license](https://img.shields.io/github/license/vividcode-ai/embedded-gdb-mcp.svg)](https://github.com/vividcode-ai/embedded-gdb-mcp/blob/main/LICENSE)

一个基于 Rust 构建的 MCP（Model Context Protocol）服务器，提供 GDB 调试功能，专为嵌入式开发工作流设计。

## 功能特性

- 启动和管理 GDB 调试会话
- 加载程序和核心转储文件进行分析
- 设置条件断点
- 单步执行代码（按行或按指令）
- 检查内存、寄存器和变量
- 查看调用栈和源代码（支持 VS Code 集成）
- 执行任意 GDB 命令
- 附加到正在运行的进程
- 为 Windows、macOS 和 Linux（x64 和 arm64）提供预编译二进制文件

## 安装

### Claude Code

```bash
claude mcp add embedded-gdb-debugger -- npx -y @vividcodeai/embedded-gdb-mcp
```

### Claude Desktop

将以下内容添加到 Claude Desktop 的 MCP 配置中：

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

### 源码编译

```bash
git clone https://github.com/vividcode-ai/embedded-gdb-mcp.git
cd embedded-gdb-mcp
cargo build --release
```

编译完成后，二进制文件位于 `target/release/embedded-gdb-mcp`。

## 使用示例

### 示例命令

#### 启动 GDB 会话

```
使用 gdb_start 启动一个新的调试会话
```

#### 加载程序

```
使用 gdb_load 将 /path/to/my/program 加载到 GDB 中，sessionId 来自 gdb_start 的返回值
```

#### 设置断点

```
使用 gdb_set_breakpoint 在 active GDB 会话的 main 函数处设置断点
```

#### 运行程序

```
使用 gdb_continue 开始执行程序
```

#### 检查变量

```
使用 gdb_print 在当前上下文中计算表达式 "my_variable" 的值
```

#### 查看调用栈

```
使用 gdb_backtrace 查看当前的调用栈
```

#### 终止会话

```
使用 gdb_terminate 结束调试会话
```

## 支持的 GDB 命令

- `gdb_start`：启动新的 GDB 会话
- `gdb_load`：加载程序到 GDB
- `gdb_command`：执行任意 GDB 命令
- `gdb_terminate`：终止 GDB 会话
- `gdb_list_sessions`：列出所有活跃的 GDB 会话
- `gdb_attach`：附加到正在运行的进程
- `gdb_load_core`：加载核心转储文件
- `gdb_set_breakpoint`：设置断点（支持条件断点）
- `gdb_continue`：继续执行程序
- `gdb_step`：单步执行（按行或按指令）
- `gdb_next`：单步跳过函数调用（按行或按指令）
- `gdb_finish`：执行到当前函数返回
- `gdb_backtrace`：显示调用栈（支持完整模式和帧数限制）
- `gdb_print`：打印表达式值
- `gdb_examine`：检查内存（支持格式和数量选项）
- `gdb_info_registers`：显示寄存器（可指定特定寄存器）
- `gdb_list_source`：在当前位置或指定位置列出源代码，支持 VS Code 集成

## 许可证

MIT

---

[English Documentation](README.md)
