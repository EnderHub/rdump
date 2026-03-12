# Release Checklist

1. Run `scripts/regenerate-generated-docs.sh`.
2. Run `cargo test --workspace`.
3. Run `cargo test -p rdump --features async`.
4. Run `cargo test -p rdump --test perf_smoke -- --ignored`.
6. Build release binaries for `rdump` and `rdump-mcp`.
7. Smoke-check `rdump --help`, `rdump search --help`, and `rdump-mcp --help`.
8. Verify no drift in `docs/generated/`.
9. If publishing from CI, ensure the crates.io publish chain is `rdump-contracts -> rdump -> rdump-mcp`.
10. Update `CHANGELOG.md` with contract-surface changes.
