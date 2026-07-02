# d2-mcp-rs

[![CI](https://github.com/Stranmor/d2-mcp-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Stranmor/d2-mcp-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/Rust-2024-orange.svg)](rust-toolchain.toml)
[![MCP](https://img.shields.io/badge/MCP-server-111827.svg)](https://modelcontextprotocol.io/)
[![D2](https://img.shields.io/badge/D2-diagrams-0f766e.svg)](https://d2lang.com/)

Render, validate, and format [D2](https://d2lang.com/) diagrams from MCP clients without giving agents arbitrary shell access.

`d2-mcp-rs` is a small Rust MCP server that wraps the official D2 CLI with typed tools, bounded file output, structured artifact reports, and default-off remote asset fetching. It is built for AI agents that need to create architecture diagrams repeatedly, safely, and in a form humans can open immediately.

![D2 MCP demo diagram](docs/assets/agent-diagram-loop.svg)

## What It Gives Agents

| Tool | Purpose | Side effect |
| --- | --- | --- |
| `d2_status` | Reports D2 availability, server limits, supported output formats, and safety policy. | None |
| `d2_layouts` | Lists layout engines available in the configured D2 CLI. | None |
| `d2_themes` | Lists themes available in the configured D2 CLI. | None |
| `d2_validate` | Validates D2 source text and returns structured diagnostics. | None |
| `d2_format` | Formats D2 source text and returns the formatted source. | None |
| `d2_render` | Renders D2 source text to SVG or PNG and returns output path, hashes, bytes, diagnostics, and optional inline SVG. | Writes only inside `D2_MCP_WORKDIR` |

## Why This Exists

AI agents can draft diagrams, but raw shell execution is the wrong interface for repeated diagram work. Agents need a narrow contract:

- no pass-through shell command tool;
- JSON schemas for every input and output;
- one configured output directory;
- path traversal rejection;
- source and render size limits;
- process timeouts;
- remote `http://` and `https://` assets blocked unless explicitly requested;
- real D2 rendering behavior because the official `d2` binary remains the renderer.

## Quick Start

Install D2 first:

```bash
curl -fsSL https://d2lang.com/install.sh | sh -s --
d2 --version
```

Install the MCP server from GitHub:

```bash
cargo install --git https://github.com/Stranmor/d2-mcp-rs.git d2-mcp
```

Create an output directory:

```bash
mkdir -p "$HOME/d2-mcp-output"
```

Add it to your MCP client:

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

Then ask your MCP client to render a diagram:

```json
{
  "source": "client -> d2_mcp -> d2_cli -> svg",
  "format": "svg",
  "output_path": "architecture/current.svg",
  "overwrite": true,
  "inline_svg": true
}
```

See [client configuration examples](docs/client-configs.md) for absolute-path setups.

## Example Diagram Source

```d2
user: User request
agent: MCP client
server: d2-mcp-rs
d2: Official D2 CLI
artifact: SVG or PNG artifact

user -> agent: asks for a diagram
agent -> server: calls d2_validate / d2_render
server -> d2: invokes bounded D2 command
d2 -> server: rendered artifact
server -> agent: structured report
agent -> user: shares final diagram
```

## Tool Details

The main tool docs are in [docs/tools.md](docs/tools.md). Short version:

- use `d2_status` before rendering to confirm the D2 binary and limits;
- use `d2_layouts` and `d2_themes` to choose valid render options without shell access;
- use `d2_validate` before a final render when the source came from a model;
- use `d2_format` when storing generated D2 source;
- use `d2_render` for final SVG/PNG output.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `D2_MCP_D2_BIN` | `d2` | D2 executable path or command name. |
| `D2_MCP_WORKDIR` | current process directory | The only directory where rendered files may be written. |
| `D2_MCP_MAX_SOURCE_BYTES` | `1048576` | Maximum D2 source input size. |
| `D2_MCP_MAX_RENDER_BYTES` | `16777216` | Maximum rendered artifact size. |

`output_path` in `d2_render` must be relative. Absolute paths and `..` components are rejected.

## Security Model

`d2-mcp-rs` narrows what an MCP client can do. It does not claim to sandbox the D2 renderer itself.

The server:

- accepts D2 source text from MCP calls;
- never exposes a generic shell command;
- never reads arbitrary user files as a tool feature;
- writes rendered artifacts only inside `D2_MCP_WORKDIR`;
- rejects absolute paths and parent-directory traversal;
- blocks remote assets by default;
- invokes the configured D2 binary with explicit arguments;
- kills the D2 process when the timeout is exceeded.

If you intentionally need remote icons or images:

```json
{
  "allow_remote_assets": true
}
```

Use that only when the workspace policy allows network fetches during rendering. See [docs/security-model.md](docs/security-model.md) for the full boundary.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The integration smoke test starts the MCP server in memory, lists the tools, checks structured output schemas, calls read-only discovery tools, renders a real SVG when D2 is installed, and verifies path traversal rejection.

## Project Status

The current goal is a dependable, small, agent-safe D2 MCP server rather than a full diagram management system. File-reading tools, project-wide diagram discovery, and diagram asset libraries should be added only as separate narrowly scoped tools with their own path policy.

## License

MIT
