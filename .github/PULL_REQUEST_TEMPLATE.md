## Summary

- 

## Verification

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test`

## Safety Boundary

- [ ] No arbitrary shell execution added.
- [ ] No arbitrary file reads added.
- [ ] Rendered output remains restricted to `D2_MCP_WORKDIR`.
- [ ] Remote assets remain blocked by default.
- [ ] New MCP inputs/outputs are typed with `serde` and `schemars`.

## Notes

- 
