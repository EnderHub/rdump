# Output Parity

This document defines the intended audience and safety behavior of each `rdump` surface.

## CLI

`summary`
- Audience: humans, shell pipelines, quick triage
- Truncation: file count only
- Content policy: no full file contents

`matches`
- Audience: humans, editor integrations
- Truncation: per-file match count may truncate
- Content policy: match text only

`snippets`
- Audience: humans, LLM snippet retrieval
- Truncation: per-file match count and snippet byte caps may truncate
- Content policy: line-ending preserving contextual snippets

`full`
- Audience: request/MCP consumers that explicitly ask for whole-file payloads
- Truncation: explicit response-level and per-item truncation fields
- Content policy: full content when allowed, with typed content-state and truncation markers

`hunks`
- Audience: humans reading merged match regions
- Truncation: formatter-level only
- Content policy: suppressed content can be shown as placeholders

`cat`
- Audience: humans, pipes
- Truncation: none beyond content-safety policy
- Content policy: full content when allowed, placeholder notice when suppressed

`find`
- Audience: humans
- Truncation: file count only
- Content policy: metadata only

`json`
- Audience: automation
- Truncation: explicit `status`, `truncated`, `truncation_reason`, and per-item truncation fields
- Content policy: full response envelope with typed diagnostics and content-state metadata

## MCP

`search`
- Audience: agents and host integrations
- Truncation: explicit response status, `next_offset`, `continuation_token`, and per-item truncation flags
- Content policy: structured output follows the same safety decisions as the SDK

`predicate_catalog`, `language_matrix`, `capability_metadata`
- Audience: agents building UIs, planners, adapters
- Truncation: none expected
- Content policy: metadata only

## SDK

`search`, `search_iter`, `search_path_iter`
- Audience: library consumers
- Truncation: exposed through `SearchReport`, `SearchResponse`, or host-selected limits
- Content policy: `content_state`, diagnostics, semantic skip reasons, and snapshot metadata are first-class
