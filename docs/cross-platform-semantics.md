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
- `display_path` is the user-facing path projection for the active backend.
- `resolved_path` is a backend-normalized stable identity path; on the real filesystem it is usually canonical, but virtual backends may supply a non-host path that is still stable within that backend.
- `root_relative_path` is present when the file can be rewritten relative to the search root.
- Path-ordering is stable and lexicographic by display path, then resolved path.
- `resolution=canonical` means the backend supplied its preferred normalized identity path; it does not require a host-filesystem canonical path.

## Snapshot identity

- Result snapshots may include `stable_token` when a backend has stronger content/version identity than size and mtime alone.
- Real-fs mode may omit `stable_token` and rely on size, mtime, and optional inode/device metadata.
- Virtual backends may use `stable_token` for drift detection even when `resolved_path` is not a host path.

## Line endings

- Structured snippets preserve their original line-ending style unless a caller explicitly requests normalization.
- Human CLI outputs may normalize or preserve line endings based on the selected line-ending mode, but the option semantics are stable across supported platforms.
