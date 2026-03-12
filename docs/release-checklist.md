# Release Checklist

1. Run `scripts/regenerate-generated-docs.sh`.
2. Run `cargo test --manifest-path rdump/Cargo.toml`.
3. Run `cargo test --manifest-path rdump/Cargo.toml --features async`.
4. Run `cargo test --manifest-path rdump-mcp/Cargo.toml`.
5. Run `cargo test --manifest-path rdump/Cargo.toml --test perf_smoke -- --ignored`.
6. Build release binaries for `rdump` and `rdump-mcp`.
7. Smoke-check `rdump --help`, `rdump search --help`, and `rdump-mcp --help`.
8. Verify no drift in `docs/generated/`.
9. Update `CHANGELOG.md` with contract-surface changes.
