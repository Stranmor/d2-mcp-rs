# Contributing

Contributions are welcome.

Before opening a pull request:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Keep the server narrow:

- do not add arbitrary shell execution;
- do not allow output paths outside `D2_MCP_WORKDIR`;
- keep all MCP inputs and outputs typed with `serde` and `schemars`;
- prefer explicit D2 CLI arguments over pass-through flag bags;
- add a regression test for every new safety boundary or bug fix.

If a feature requires reading project files, add a separate narrowly scoped tool and document the exact path policy instead of expanding the existing source-text tools.
