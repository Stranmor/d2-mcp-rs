# Security Model

`d2-mcp-rs` is a bounded MCP adapter around the official D2 CLI. It is not a full sandbox.

## Trust Boundary

The MCP client supplies D2 source text. The server validates size, path policy, timeout, and remote asset policy before invoking D2.

The server does not expose:

- arbitrary shell execution;
- arbitrary file reads;
- unrestricted output paths;
- pass-through D2 flag bags.

## File Writes

`d2_render` writes only inside `D2_MCP_WORKDIR`.

The following are rejected:

- absolute output paths;
- `..` path components;
- empty output paths;
- existing files unless `overwrite=true`.

If no output path is supplied, the server writes under `.d2-mcp-output/` inside `D2_MCP_WORKDIR`.

## Remote Assets

Remote `http://` and `https://` references are blocked by default because D2 can fetch icons or images during rendering.

Enable remote references only for a specific call:

```json
{
  "allow_remote_assets": true
}
```

Use an OS or container sandbox if untrusted users can submit D2 source.

## Process Limits

The server enforces:

- maximum source bytes through `D2_MCP_MAX_SOURCE_BYTES`;
- maximum rendered output bytes through `D2_MCP_MAX_RENDER_BYTES`;
- per-call timeout with a hard upper bound.

If D2 exceeds the configured timeout, the child process is killed and the tool returns a timeout error.

## Recommended Deployment

For local personal use, a dedicated output directory is usually enough.

For shared or untrusted use:

- run the MCP server under a low-privilege OS user;
- point `D2_MCP_WORKDIR` to a dedicated writable directory;
- place the server inside a container or sandbox;
- keep remote assets disabled unless explicitly required;
- monitor disk usage for the output directory.
