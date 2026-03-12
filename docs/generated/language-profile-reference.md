# Language Semantic Profile Reference

Generated from live tree-sitter profiles. Capture convention: `@match`.

## Bash (sh)

- Support tier: `stable`
- Aliases: `bash, sh`
- Extensions: `sh, bash`
- Semantic predicates: `call, comment, def, func, import, str`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## C (c)

- Support tier: `stable`
- Aliases: `c`
- Extensions: `c, h`
- Semantic predicates: `call, comment, def, enum, func, import, macro, str, struct, type`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `enum`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `macro`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `struct`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## C# (cs)

- Support tier: `stable`
- Aliases: `cs, csx`
- Extensions: `cs, csx`
- Semantic predicates: `call, class, comment, def, enum, func, import, interface, str, struct, type`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `enum`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `interface`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `struct`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## C++ (cpp)

- Support tier: `stable`
- Aliases: `cc, cpp, cxx, hh, hpp, hxx`
- Extensions: `cpp, cc, cxx, hpp, hh, hxx`
- Semantic predicates: `call, class, comment, def, enum, func, import, macro, str, struct`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `enum`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `macro`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `struct`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## CSS (css)

- Support tier: `partial`
- Aliases: `css`
- Extensions: `css`
- Semantic predicates: `call, comment, def, import, str, type`
- Caveats:
  - CSS semantic coverage is partial and focuses on selectors and declarations, not cascade resolution.
  - Support tier is partial; some language constructs may not produce semantic captures yet.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## Elixir (ex)

- Support tier: `stable`
- Aliases: `ex, exs`
- Extensions: `ex, exs`
- Semantic predicates: `call, comment, def, func, import, module, protocol, str`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `module`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `protocol`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## Go (go)

- Support tier: `stable`
- Aliases: `go`
- Extensions: `go`
- Semantic predicates: `call, comment, def, func, import, interface, str, struct, type`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `interface`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `struct`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## HTML (html)

- Support tier: `partial`
- Aliases: `html`
- Extensions: `html, htm`
- Semantic predicates: `call, comment, def, import, str`
- Caveats:
  - HTML semantic coverage is partial and focuses on structural nodes rather than browser/runtime behavior.
  - Support tier is partial; some language constructs may not produce semantic captures yet.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## Haskell (hs)

- Support tier: `experimental`
- Aliases: `hs, lhs`
- Extensions: `hs, lhs`
- Semantic predicates: `call, comment, def, func, import, module, str, type`
- Caveats:
  - This profile is experimental; expect narrower predicate coverage and fewer regression fixtures.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `module`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## Java (java)

- Support tier: `stable`
- Aliases: `java`
- Extensions: `java`
- Semantic predicates: `call, class, comment, def, enum, func, import, interface, str`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `enum`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `interface`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## JavaScript (js)

- Support tier: `stable`
- Aliases: `js`
- Extensions: `js`
- Semantic predicates: `call, class, comment, customhook, def, func, hook, import, str`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `customhook`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode=wildcard` enables shell-style `*` matching.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `hook`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode=wildcard` enables shell-style `*` matching.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## Lua (lua)

- Support tier: `stable`
- Aliases: `lua`
- Extensions: `lua`
- Semantic predicates: `call, comment, def, func, import, str`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## OCaml (ml)

- Support tier: `experimental`
- Aliases: `ml, mli`
- Extensions: `ml, mli`
- Semantic predicates: `call, comment, def, func, import, module, str, type`
- Caveats:
  - This profile is experimental; expect narrower predicate coverage and fewer regression fixtures.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `module`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## PHP (php)

- Support tier: `stable`
- Aliases: `php`
- Extensions: `php, phtml`
- Semantic predicates: `call, class, comment, def, func, import, interface, str, trait`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `interface`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `trait`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## Python (py)

- Support tier: `stable`
- Aliases: `py`
- Extensions: `py`
- Semantic predicates: `call, class, comment, def, func, import, str`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## React (jsx)

- Support tier: `stable`
- Aliases: `jsx, tsx`
- Extensions: `jsx, tsx`
- Semantic predicates: `comment, component, customhook, element, hook, import, prop, str`
- Caveats:
  - React-specific predicates are only available on JSX/TSX profiles and remain more permissive than language-core predicates.

### Matching Rules

- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `component`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `customhook`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode=wildcard` enables shell-style `*` matching.
- `element`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `hook`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode=wildcard` enables shell-style `*` matching.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `prop`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## Ruby (rb)

- Support tier: `stable`
- Aliases: `rb`
- Extensions: `rb`
- Semantic predicates: `call, class, comment, def, func, import, module, str, type`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `module`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## Rust (rs)

- Support tier: `stable`
- Aliases: `rs`
- Extensions: `rs`
- Semantic predicates: `call, comment, def, enum, func, impl, import, macro, module, str, struct, trait, type`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `enum`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `impl`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `macro`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `module`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `struct`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `trait`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## SQL (Generic) (sql)

- Support tier: `partial`
- Aliases: `sql`
- Extensions: `sql`
- Semantic predicates: `call, comment, def, func, import, str`
- Caveats:
  - SQL dialect selection is heuristic unless overridden; enable strict mode to fail instead of falling back.
  - Support tier is partial; some language constructs may not produce semantic captures yet.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## SQL (MySQL) (sqlmysql)

- Support tier: `stable`
- Aliases: `sqlmysql`
- Extensions: `mysql`
- Semantic predicates: `call, comment, def, func, import, str`
- Caveats:
  - SQL dialect selection is heuristic unless overridden; enable strict mode to fail instead of falling back.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## SQL (Postgres) (sqlpg)

- Support tier: `stable`
- Aliases: `sqlpg`
- Extensions: `psql, pgsql`
- Semantic predicates: `call, comment, def, func, import, str`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## SQL (SQLite) (sqlsqlite)

- Support tier: `stable`
- Aliases: `sqlsqlite`
- Extensions: `sqlite`
- Semantic predicates: `call, comment, def, func, import, str`
- Caveats:
  - SQL dialect selection is heuristic unless overridden; enable strict mode to fail instead of falling back.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## Scala (scala)

- Support tier: `experimental`
- Aliases: `scala`
- Extensions: `scala`
- Semantic predicates: `call, class, comment, def, func, import, object, str, trait, type`
- Caveats:
  - This profile is experimental; expect narrower predicate coverage and fewer regression fixtures.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `object`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `trait`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## Swift (swift)

- Support tier: `experimental`
- Aliases: `swift`
- Extensions: `swift`
- Semantic predicates: `call, class, comment, def, func, import, protocol, str`
- Caveats:
  - This profile is experimental; expect narrower predicate coverage and fewer regression fixtures.

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `protocol`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.

## TypeScript (ts)

- Support tier: `stable`
- Aliases: `ts`
- Extensions: `ts`
- Semantic predicates: `call, class, comment, customhook, def, enum, func, hook, import, interface, str, type`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `class`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `customhook`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode=wildcard` enables shell-style `*` matching.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `enum`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `hook`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode=wildcard` enables shell-style `*` matching.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `interface`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

## Zig (zig)

- Support tier: `stable`
- Aliases: `zig`
- Extensions: `zig`
- Semantic predicates: `call, comment, def, enum, func, import, str, struct, type`
- Caveats: none recorded

### Matching Rules

- `call`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `comment`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `def`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `enum`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `func`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `import`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `str`: Substring match against captured text by default. `semantic_match_mode` can tighten this to exact, prefix, regex, or wildcard semantics.
- `struct`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.
- `type`: Exact match by default, with `.` accepted as the broad wildcard. `semantic_match_mode` can switch to case-insensitive, prefix, regex, or wildcard behavior.

