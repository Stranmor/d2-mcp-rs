# d2-mcp-rs

Rust MCP server for generating [D2](https://d2lang.com/) diagrams from AI agents.

It gives an MCP-capable client four tools:

- `d2_status` checks the local D2 CLI and server limits.
- `d2_validate` validates D2 source text.
- `d2_format` formats D2 source text without mutating user files.
- `d2_render` renders D2 source text to SVG or PNG and returns a structured artifact report.

The server wraps the official `d2` CLI instead of reimplementing the D2 language. That keeps rendering behavior aligned with D2 itself while giving agents a typed, bounded interface.

## Why Use It

AI agents are good at drafting diagrams, but raw shell access is a poor interface for repeated diagram work. This server gives agents a narrow tool surface:

- typed JSON schemas for every input and output;
- SVG/PNG rendering through the official D2 renderer;
- output files restricted to one configured working directory;
- process timeouts and input/output size limits;
- remote `http://` / `https://` assets blocked by default;
- no arbitrary shell command execution.

## Requirements

- Rust 2024 toolchain
- D2 CLI available on `PATH`

Install D2 with your preferred package manager or follow the official D2 install docs:

```bash
curl -fsSL https://d2lang.com/install.sh | sh -s --
```

Check it:

```bash
d2 --version
```

## Install

From source:

```bash
git clone https://github.com/Stranmor/d2-mcp-rs.git
cd d2-mcp-rs
cargo build --release
```

The binary will be at:

```bash
target/release/d2-mcp
```

## MCP Client Config

Example stdio config:

```json
{
  "mcpServers": {
    "d2": {
      "command": "/absolute/path/to/d2-mcp",
      "env": {
        "D2_MCP_WORKDIR": "/absolute/path/for/rendered-diagrams"
      }
    }
  }
}
```

Optional environment variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `D2_MCP_D2_BIN` | `d2` | D2 executable path or command name. |
| `D2_MCP_WORKDIR` | current process directory | Only directory where rendered files may be written. |
| `D2_MCP_MAX_SOURCE_BYTES` | `1048576` | Maximum D2 source input size. |
| `D2_MCP_MAX_RENDER_BYTES` | `16777216` | Maximum rendered artifact size. |

## Example Tool Calls

Validate:

```json
{
  "source": "client -> d2_mcp -> svg"
}
```

Render SVG:

```json
{
  "source": "client -> d2_mcp -> svg",
  "format": "svg",
  "output_path": "architecture/current.svg",
  "overwrite": true,
  "inline_svg": true
}
```

Render PNG with a theme:

```json
{
  "source": "frontend -> api -> database",
  "format": "png",
  "output_path": "architecture/current.png",
  "theme": 300,
  "layout": "elk"
}
```

`output_path` must be relative and must stay inside `D2_MCP_WORKDIR`.

## Security Model

`d2-mcp-rs` is intentionally small:

- it accepts D2 source text from the MCP call;
- it does not read arbitrary user files;
- it writes rendered output only inside `D2_MCP_WORKDIR`;
- it rejects `..` and absolute output paths;
- it blocks remote asset references by default;
- it invokes only the configured D2 binary with explicit arguments;
- it kills D2 if the configured timeout is exceeded.

If a diagram intentionally needs remote images or icons, pass:

```json
{
  "allow_remote_assets": true
}
```

Use that only when the MCP client and workspace policy allow outbound network fetches during rendering.

## Development

Run the local checks:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The integration smoke test starts the server in memory, lists MCP tools, validates output schemas, and renders a real SVG when the `d2` binary is installed.

## License

MIT
