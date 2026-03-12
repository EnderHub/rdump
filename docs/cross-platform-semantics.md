# Cross-Platform Semantics

This document records the portability rules that machine consumers should rely on.

## Hidden files and ignore rules

- Default search respects `.gitignore`, `.rdumpignore`, and built-in ignore patterns on every platform.
- `--hidden` enables hidden-file discovery; hidden-file naming is platform-specific, but the flag behavior is stable.
- `--no-ignore` disables ignore sources and should be treated as the portable override.

## `find` metadata

- Structured `find`/path JSON always includes `size_bytes`, `modified_unix_millis`, `readonly`, and `permissions_display`.
- `permissions_display` is a Unix mode string on Unix hosts.
- `permissions_display` falls back to `readonly` / `readwrite` semantics on non-Unix hosts.

## Paths

- Machine-facing results expose both `display_path` and `resolved_path`.
- `root_relative_path` is present when the file can be rewritten relative to the search root.
- Path-ordering is stable and lexicographic by display path, then resolved path.

## Line endings

- Structured snippets preserve their original line-ending style unless a caller explicitly requests normalization.
- Human CLI outputs may normalize or preserve line endings based on the selected line-ending mode, but the option semantics are stable across supported platforms.
