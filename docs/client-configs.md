# MCP Client Configuration

The server uses stdio transport. The safest setup is to use an absolute binary path and an explicit `D2_MCP_WORKDIR`.

## Local `cargo install`

```bash
cargo install --git https://github.com/Stranmor/d2-mcp-rs.git d2-mcp
mkdir -p "$HOME/d2-mcp-output"
```

MCP config:

```json
{
  "mcpServers": {
    "d2": {
      "command": "d2-mcp",
      "env": {
        "D2_MCP_WORKDIR": "/home/you/d2-mcp-output"
      }
    }
  }
}
```

## Local Source Checkout

```bash
git clone https://github.com/Stranmor/d2-mcp-rs.git
cd d2-mcp-rs
cargo build --release
mkdir -p "$HOME/d2-mcp-output"
```

MCP config:

```json
{
  "mcpServers": {
    "d2": {
      "command": "/home/you/d2-mcp-rs/target/release/d2-mcp",
      "env": {
        "D2_MCP_WORKDIR": "/home/you/d2-mcp-output"
      }
    }
  }
}
```

## Custom D2 Binary

Use `D2_MCP_D2_BIN` when D2 is not on `PATH` or when a pinned D2 binary should be used.

```json
{
  "mcpServers": {
    "d2": {
      "command": "/home/you/.cargo/bin/d2-mcp",
      "env": {
        "D2_MCP_D2_BIN": "/home/you/bin/d2",
        "D2_MCP_WORKDIR": "/home/you/d2-mcp-output"
      }
    }
  }
}
```

## Readiness Check

After connecting the MCP client, call `d2_status`.

Expected state:

```json
{
  "status": "ready",
  "reads_arbitrary_files": false,
  "writes_outside_workdir": false
}
```

If `status` is `unavailable`, the MCP server started but could not run the configured D2 binary.
