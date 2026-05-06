# @vividcodeai/embedded-gdb-mcp

通过 MCP 协议提供 GDB 调试功能的服务器。

## 安装

```bash
npm install -g @vividcodeai/embedded-gdb-mcp
```

## 使用

在 MCP 客户端中配置：

```json
{
  "mcpServers": {
    "embedded-gdb-debugger": {
      "command": "embedded-gdb-mcp"
    }
  }
}
```

完整文档: [github.com/vividcode-ai/embedded-gdb-mcp](https://github.com/vividcode-ai/embedded-gdb-mcp)

---

[English Documentation](README.md)
