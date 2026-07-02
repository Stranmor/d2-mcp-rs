# Tool Reference

`d2-mcp-rs` exposes six MCP tools. All tools return structured content and JSON-schema output metadata through the MCP server.

## `d2_status`

Reports server readiness, D2 CLI version, configured limits, supported output formats, and safety policy.

Use it when an agent first connects or before claiming that diagram rendering is available.

## `d2_layouts`

Lists layout engines available to the configured D2 CLI.

This is read-only discovery. It lets an agent choose a valid `layout` value for `d2_render` without direct shell access.

## `d2_themes`

Lists D2 themes available to the configured D2 CLI.

This is read-only discovery. It lets an agent choose valid `theme` and `dark_theme` values for `d2_render` without direct shell access.

## `d2_validate`

Input:

```json
{
  "source": "client -> server",
  "allow_remote_assets": false,
  "timeout_seconds": 20
}
```

Output includes:

- `status`: `valid` or `invalid`;
- source byte count;
- source SHA-256;
- elapsed time;
- D2 version;
- diagnostics from D2.

Invalid D2 source is returned as structured validation state rather than as a transport failure.

## `d2_format`

Input:

```json
{
  "source": "client -> server",
  "allow_remote_assets": false,
  "timeout_seconds": 20
}
```

Output includes:

- formatted source;
- whether it changed;
- formatted byte count and SHA-256;
- elapsed time;
- diagnostics.

The tool writes the source to a temporary file for `d2 fmt` and returns the formatted source. It does not mutate user files.

## `d2_render`

Input:

```json
{
  "source": "client -> d2_mcp -> svg",
  "format": "svg",
  "output_path": "architecture/current.svg",
  "overwrite": true,
  "inline_svg": true,
  "allow_remote_assets": false,
  "theme": 300,
  "dark_theme": -1,
  "layout": "elk",
  "sketch": false,
  "pad": 100,
  "timeout_seconds": 20
}
```

Required fields:

- `source`;
- `format`: `svg` or `png`.

Optional render controls:

- `output_path`: relative path inside `D2_MCP_WORKDIR`;
- `overwrite`: required when the target file already exists;
- `inline_svg`: returns SVG text inline when the rendered SVG is small enough;
- `allow_remote_assets`: enables remote asset references for this call;
- `theme` and `dark_theme`: D2 theme IDs;
- `layout`: ASCII layout engine name;
- `sketch`;
- `pad`;
- `timeout_seconds`.

Output includes:

- rendered output path relative to `D2_MCP_WORKDIR`;
- output bytes and SHA-256;
- source bytes and SHA-256;
- optional inline SVG;
- elapsed time;
- D2 version;
- diagnostics.

## Path Policy

`output_path` must be relative and must stay inside `D2_MCP_WORKDIR`.

Rejected examples:

```text
/tmp/diagram.svg
../diagram.svg
architecture/../../diagram.svg
```

Accepted examples:

```text
architecture/current.svg
.d2-mcp-output/generated.svg
```
