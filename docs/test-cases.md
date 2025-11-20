# Test Cases

Comprehensive collection of rdump query examples and use cases for test coverage.

---

## Metadata Predicates

### ext (Extension)

```bash
# Basic extension matching (case-insensitive)
rdump "ext:rs"
rdump "ext:RS"           # Should match .rs files
rdump "ext:py"
rdump "ext:js"
rdump "ext:jsx"
rdump "ext:ts"
rdump "ext:tsx"
rdump "ext:go"
rdump "ext:java"
rdump "ext:toml"
rdump "ext:yml"
rdump "ext:log"
rdump "ext:tmp"

# Multiple extensions with OR
rdump "ext:toml | ext:yml"
rdump "ext:js | ext:tsx"
rdump "ext:js | ext:jsx"
rdump "ext:js | ext:ts"
```

### name (Glob Pattern)

```bash
# Glob patterns on filename
rdump "name:*_test.rs"
rdump "name:*.config.js"
rdump "name:*.spec.ts"
rdump "name:*.log"
rdump "name:*.txt"
rdump "name:README*"
rdump "name:Dockerfile"
rdump "name:readme.md"              # Case-insensitive
rdump "name:test_*.rs"
rdump "name:?oo.txt"                # Single character wildcard
rdump "name:[abc]*.rs"              # Character class
rdump "name:main.rs"
rdump "name:utils.rs"
```

### path (Substring/Glob Match)

```bash
# Path substring matching
rdump "path:src/components"
rdump "path:src/api/"
rdump "path:tests/"
rdump "path:components"
rdump "path:models/"
rdump "path:/db/"
rdump "path:/repository/"
rdump "path:old"
rdump "path:build_info"

# Globstar patterns
rdump "path:**/*.rs"
```

### in (Directory Scope)

```bash
# Current directory only
rdump "in:. & name:*.rs"

# Specific directories
rdump "in:src & struct:User"
rdump "in:tests"
rdump "in:benches"
rdump "in:docs & func:test_user"
```

### path_exact (Exact Path Match)

```bash
rdump "path_exact:/absolute/path/to/file.rs"
```

### size (File Size)

```bash
# Size comparisons
rdump "size:>100kb"
rdump "size:<1000"              # Bytes
rdump "size:>2kb"
rdump "size:<1mb"
rdump "size:>1gb"
rdump "size:>50kb"
rdump "size:>10kb"
rdump "size:>1kb"
rdump "size:>40b"
rdump "size:<40b"

# Exact size
rdump "size:=123"
rdump "size:=0"                 # Empty files

# Combined
rdump "ext:go & size:>50kb"
rdump "ext:toml & size:>1kb & modified:<2d"
rdump "import:old_module & size:>2kb"
rdump "contains:TODO & size:<1000"
```

### modified (Modification Time)

```bash
# Time comparisons
rdump "modified:<2d"            # Last 48 hours
rdump "modified:<1h"            # Last hour
rdump "modified:>7d"            # Older than a week
rdump "modified:<30m"           # Last 30 minutes
rdump "modified:<1w"            # Last week
rdump "modified:>1s"            # More than 1 second ago
rdump "modified:<1s"            # Less than 1 second ago

# Exact date
rdump "modified:2024-11-19"

# Combined
rdump "ext:tmp & modified:>7d"
rdump "ext:toml & size:>1kb & modified:<2d"
rdump "size:>10kb & modified:<7d & (contains:TODO | contains:FIXME)"
```

---

## Content Predicates

### contains / c (Literal Substring)

```bash
# Case-sensitive literal search
rdump "contains:fn main()"
rdump "c:'fn main()'"
rdump "contains:database"
rdump "contains:TODO"
rdump "contains:useState"
rdump "contains:foo"
rdump "contains:error"
rdump "contains:warn"
rdump "contains:main"
rdump "contains:utility"
rdump "contains:hello"
rdump "contains:match"
rdump "contains:main function"
rdump "contains:MAIN FUNCTION"
rdump "contains:this should not be found"
rdump "contains:This should be ignored."
rdump "contains:.await"

# Special characters
rdump "contains:'value * 2'"
rdump "contains:\"user's settings\""

# Combined
rdump "(ext:toml | ext:yml) & contains:database"
rdump "ext:rs & contains:fn"
rdump "name:*.log & contains:error"
rdump "contains:main and ext:rs"
```

### matches / m (Regex)

```bash
# Regular expression search
rdump "m:'/struct \\w+/'"
rdump "matches:'fn\\s+\\w+'"
rdump "m:'import\\s+.*from'"
rdump "matches:'TODO:.*'"
rdump "matches:'(hello|world)'"
rdump "matches:'\\(hello world\\)'"      # Escaped parens
rdump "matches:'hello world'"
rdump "matches:'goodbye world'"
rdump "matches:'(?i)hello world'"        # Case-insensitive
rdump "matches:'こんにちは世界'"           # Unicode
rdump "matches:'('"                       # Invalid regex (should error)

# Regex in security queries
rdump "str:/[A-Za-z0-9_\\-]{20,}/ & !path:test"
rdump "str:/SELECT.*FROM/ & !(path:/db/ | path:/repository/)"
rdump "name:Dockerfile & !contains:/@sha256:/"
```

---

## Code-Aware Predicates

### def (Generic Definition)

```bash
rdump "def:Cli"
rdump "def:Cli & ext:rs"
rdump "def:User"
rdump "def:Order"
rdump "def:Database"
rdump "def:Role"
rdump "def:Helper & ext:py"
rdump "def:OldLogger"
rdump "def:ILog | def:LogLevel"
rdump "def:NonExistent"
rdump "def:NonExistent & ext:py"
rdump "def:NonExistent & (ext:js | ext:ts)"

# Combined
rdump "path:src/api/ & (def:User | def:Order)"
rdump "def:Role | def:User"
```

### struct (Struct Definition)

```bash
rdump "struct:User"
rdump "struct:Cli"
rdump "struct:Config & ext:rs"
rdump "struct:Server & ext:go"
rdump "struct:Point"
rdump "struct:NonExistent & ext:go"

# Wildcard - match any struct
rdump "struct:."
rdump "struct:. & func:. & ext:rs"

# Combined
rdump "ext:rs & struct:User"
rdump "path:models/ & ext:rs & struct:User"
rdump "struct:Cli & comment:TODO"
rdump "struct:User & !comment:TODO"
rdump "struct:Order & comment:TODO"
rdump "type:UserId & struct:User"
```

### class (Class Definition)

```bash
rdump "class:Application & ext:java"
rdump "class:Helper"
rdump "class:NonExistent & ext:java"
```

### enum (Enum Definition)

```bash
rdump "enum:Status"
rdump "def:Role"                    # Enums via def
```

### interface (Interface Definition)

```bash
rdump "interface:ILog"
rdump "interface:ILog & type:LogLevel"
rdump "interface:Name"
```

### trait (Trait Definition)

```bash
rdump "trait:Summary"
rdump "trait:Name"
```

### type (Type Alias)

```bash
rdump "type:UserId"
rdump "type:LogLevel"
rdump "type:Alias"
rdump "type:UserId & struct:User"
rdump "interface:ILog & type:LogLevel"
```

### impl (Implementation Block)

```bash
rdump "impl:NewsArticle"
rdump "impl:Name"
```

### func (Function Definition)

```bash
rdump "func:main"
rdump "func:new"
rdump "func:process_data"
rdump "func:process_data & ext:py"
rdump "func:run_helper"
rdump "func:createLog"
rdump "func:NewServer"
rdump "func:test_user"
rdump "func:non_existent_function"
rdump "func:name"

# Wildcard - match any function
rdump "func:."
rdump "ext:py & func:."
rdump "ext:js & !func:."
rdump "struct:. & func:. & ext:rs"

# Combined
rdump "ext:rs & func:main"
rdump "func:main & ext:rs"          # Suboptimal order
rdump "func:main & ext:py"
rdump "func:main | import:serde"
rdump "func:main & call:println"
rdump "func:NewServer | call:NewServer"
```

### macro (Macro Definition)

```bash
rdump "macro:my_macro"
rdump "macro:name"
```

### import (Import Statement)

```bash
rdump "import:react"
rdump "import:serde"
rdump "import:os & ext:py"
rdump "import:path & ext:ts"
rdump "import:fmt"
rdump "import:ArrayList"
rdump "import:old_module"
rdump "import:module"
rdump "import:std"
rdump "import:'@scope/pkg'"

# Wildcard - match any import
rdump "ext:py & !import:."

# Combined
rdump "import:react & (ext:js | ext:tsx)"
rdump "import:react & ext:tsx & path:components"
rdump "import:old_module & size:>2kb"
rdump "import:fmt & comment:\"HTTP server\""
rdump "import:ArrayList & comment:HACK"
rdump "func:main | import:serde"
rdump "--format=paths import:react & (ext:js | ext:tsx)"
```

### call (Function/Macro Call)

```bash
rdump "call:process_data"
rdump "call:useState"
rdump "call:println"
rdump "call:my_func"
rdump "call:my_macro"
rdump "call:log & ext:js"
rdump "call:log & ext:ts"
rdump "call:run_helper | call:do_setup"
rdump "call:process_payment"
rdump "call:NewServer"
rdump "call:user_service"

# Combined
rdump "call:println & ext:rs"
rdump "call:my_func & path:same_file_def_call.rs"
rdump "func:NewServer | call:NewServer"
rdump "func:main & call:println"
rdump "ext:rs & call:user_service & !contains:.await"
```

### comment (Comment Content)

```bash
rdump "comment:TODO"
rdump "comment:FIXME"
rdump "comment:HACK"
rdump "comment:REVIEW"
rdump "comment:deprecated"
rdump "comment:ignore"
rdump "comment:skip"
rdump "comment:\"HTTP server\""
rdump "comment:\"A JSX comment\""

# Combined
rdump "struct:Cli & comment:TODO"
rdump "struct:User & !comment:TODO"
rdump "class:Helper & comment:FIXME"
rdump "import:ArrayList & comment:HACK"
rdump "import:fmt & comment:\"HTTP server\""
rdump "(comment:ignore | comment:skip) & name:*test*"
```

### str (String Literal)

```bash
rdump "str:error"
rdump "str:'Hello, World'"
rdump "str:\"Hello, world!\""
rdump "str:\"Hello from Java!\""
rdump "str:/tmp/data"
rdump "str:logging:"
rdump "str:literal"

# Regex in string literals
rdump "str:/[A-Za-z0-9_\\-]{20,}/ & !path:test"
rdump "str:/SELECT.*FROM/ & !(path:/db/ | path:/repository/)"
```

---

## React-Specific Predicates

### component (Component Definition)

```bash
rdump "component:App & ext:tsx"
rdump "component:Button & ext:jsx"
rdump "component:ClassComponent"
rdump "component:MemoizedComponent"
rdump "component:Component"

# Combined
rdump "path:react_comprehensive.tsx & component:ClassComponent"
rdump "path:react_comprehensive.tsx & component:MemoizedComponent"
rdump "ext:tsx & !component:ClassComponent"
rdump "component:MemoizedComponent & hook:useMemo"
```

### element (JSX Element)

```bash
rdump "element:h1 & ext:tsx"
rdump "element:Button & ext:tsx"
rdump "element:div"
rdump "element:input"
rdump "element:button"
rdump "element:ClassComponent"
rdump "element:SVG.Circle"              # Namespaced

# Combined
rdump "element:Button & !prop:disabled"
rdump "element:Button & prop:disabled"
rdump "element:input & prop:id"
rdump "element:button & prop:disabled"
rdump "path:react_comprehensive.tsx & element:input & prop:id"
rdump "path:react_comprehensive.tsx & element:ClassComponent"
rdump "path:react_comprehensive.tsx & element:SVG.Circle"
```

### hook (Hook Call)

```bash
rdump "hook:useState"
rdump "hook:useEffect"
rdump "hook:useCallback"
rdump "hook:useMemo"
rdump "hook:useAuth"

# Wildcard - match any hook
rdump "hook:."
rdump "path:react_comprehensive.tsx & hook:."

# Combined
rdump "ext:tsx & hook:useState & !hook:useCallback"
rdump "component:MemoizedComponent & hook:useMemo"
rdump "path:react_comprehensive.tsx & hook:useEffect"
```

### customhook (Custom Hook Definition)

```bash
rdump "customhook:useAuth"
rdump "customhook:useWindowWidth"

# Wildcard - match any custom hook
rdump "customhook:."
rdump "path:react_comprehensive.tsx & customhook:."
rdump "path:react_comprehensive.tsx & customhook:useWindowWidth"
```

### prop (JSX Prop)

```bash
rdump "prop:onClick"
rdump "prop:disabled"
rdump "prop:id"
rdump "prop:name"

# Combined
rdump "element:Button & prop:disabled"
rdump "element:Button & !prop:disabled"
rdump "element:input & prop:id"
rdump "element:button & prop:disabled"
rdump "path:react_comprehensive.tsx & element:input & prop:id"
rdump "path:react_comprehensive.tsx & element:button & prop:disabled"
```

---

## Logical Operators

### AND (& or AND)

```bash
rdump "ext:rs & contains:fn"
rdump "ext:py & func:main"
rdump "path:src & size:>1kb"
rdump "ext:rs & (contains:'foo' | name:'*_test.rs')"
rdump "import:react & ext:tsx & path:components"
rdump "struct:MyStruct & ext:rs & path:code.rs"

# Named operator (case-insensitive)
rdump "name:*.log AND contains:error"
rdump "contains:main and ext:rs"
```

### OR (| or OR)

```bash
rdump "ext:toml | ext:yml"
rdump "ext:js | ext:tsx"
rdump "def:User | def:Order"
rdump "contains:'foo' | name:'*_test.rs'"
rdump "def:ILog | def:LogLevel"
rdump "func:main | import:serde"
rdump "call:run_helper | call:do_setup"
rdump "func:NewServer | call:NewServer"

# Named operator
rdump "name:*.log or name:*.txt"
```

### NOT (! or NOT)

```bash
rdump "ext:rs & !path:tests"
rdump "!ext:md"
rdump "contains:TODO & !path:vendor"
rdump "struct:User & !comment:TODO"
rdump "ext:tsx & !component:ClassComponent"
rdump "ext:js & !func:."
rdump "ext:py & !import:."

# Negated content
rdump "ext:rs & call:user_service & !contains:.await"

# Named operator
rdump "not path:old"
```

### Grouping with Parentheses

```bash
rdump "ext:rs & (contains:'foo' | name:'*_test.rs')"
rdump "(ext:toml | ext:yml) & contains:database"
rdump "path:src/api/ & (def:User | def:Order)"
rdump "import:react & (ext:js | ext:tsx)"
rdump "(ext:rs | ext:py) & (func:main | func:test)"
rdump "(name:main.rs | name:utils.rs) & contains:hello"
rdump "(contains:main | contains:utility) & ext:rs"
rdump "contains:foo | !contains:baz"
rdump "(ext:ts | ext:css) & !name:*.spec.ts"
rdump "def:NonExistent & (ext:js | ext:ts)"

# Negated groups
rdump "ext:rs & !(path:tests | path:benches)"
rdump "ext:rs & !(in:tests | in:benches)"
rdump "str:/SELECT.*FROM/ & !(path:/db/ | path:/repository/)"
```

### Complex/Nested Combinations

```bash
# Deeply nested
rdump "((name:*.log or name:*.txt) and (contains:error or contains:warn)) and not (path:old)"
rdump "((ext:rs | ext:py) & (func:main | func:test)) & !path:vendor"
rdump "((ext:js | ext:ts) & import:react) | (ext:tsx & contains:useState)"

# Multiple semantic predicates
rdump "ext:rs & func:new & import:std"
rdump "(def:User | def:Account) & path:models"
rdump "type:UserId & struct:User"
rdump "interface:ILog & type:LogLevel"
rdump "struct:. & func:. & ext:rs"

# Mixed content and semantic
rdump "size:>10kb & modified:<7d & (contains:TODO | contains:FIXME)"
rdump "component:MemoizedComponent & hook:useMemo"
rdump "ext:rs & call:user_service & !contains:.await"
```

---

## Output Formats

### Markdown (Default)

```bash
rdump "ext:rs"
rdump --format=markdown "ext:rs"
```

### JSON

```bash
rdump --format=json "def:Database"
rdump --format=json "ext:rs & func:main"
rdump --format=json "import:react & ext:tsx"
```

### Cat (Concatenated Content)

```bash
rdump --format=cat "ext:rs"
```

### Paths Only

```bash
rdump --format=paths "import:react & (ext:js | ext:tsx)"
rdump --format=paths "ext:rs"
rdump --format=paths "ext:tsx & path:components"
```

---

## CLI Flags

### Line Numbers

```bash
rdump --line-numbers "ext:rs"
rdump -n "ext:rs & contains:fn"
```

### No Headers

```bash
rdump --no-headers "ext:rs"
rdump --no-headers --line-numbers "ext:rs"
```

### Ignore Files

```bash
rdump --no-ignore "ext:rs"
rdump --hidden "ext:rs"
```

### Directory Options

```bash
rdump --root /path/to/dir "ext:rs"
rdump --max-depth 2 "ext:rs"
```

### Output to File

```bash
rdump -o context.txt "path:src/api/ & (def:User | def:Order)"
rdump --output results.json --format=json "ext:rs"
```

### Verbose Mode

```bash
rdump --verbose "ext:rs & func:main"
rdump -v "def:User"
```

### Threads

```bash
rdump --threads 4 "ext:rs"
```

---

## Real-World Use Cases

### Security Auditing

```bash
# Find potential hardcoded secrets
rdump "str:/[A-Za-z0-9_\\-]{20,}/ & !path:test"

# Find raw SQL queries outside db packages
rdump "str:/SELECT.*FROM/ & !(path:/db/ | path:/repository/)"

# Dockerfiles without pinned images
rdump "name:Dockerfile & !contains:/@sha256:/"

# Locate disabled tests
rdump "(comment:ignore | comment:skip) & name:*test*"
```

### Context Grabbing for LLMs

```bash
# Grab API context
rdump "path:src/api/ & (def:User | def:Order)" > context.txt

# Find all database-related config
rdump "(ext:toml | ext:yml) & contains:database"

# Impact analysis before refactor
rdump "import:old_module & size:>2kb"

# Find call sites
rdump "call:process_payment"
```

### Codebase Exploration

```bash
# Find all test files
rdump "name:*_test.rs"
rdump "name:*.spec.js"

# Find large files modified recently
rdump "size:>100kb & modified:<7d"
rdump "ext:go & size:>50kb"

# Find TODO comments in source
rdump "path:src & comment:TODO"

# Find all functions in Python files
rdump "ext:py & func:."

# Find files with no functions
rdump "ext:js & !func:."

# Find files with no imports
rdump "ext:py & !import:."

# Temporary files older than a week
rdump "ext:tmp & modified:>7d"
```

### React Development

```bash
# Components with useState but not useCallback
rdump "ext:tsx & hook:useState & !hook:useCallback"

# List all custom hooks
rdump "customhook:."

# Find Button elements missing disabled prop
rdump "element:Button & !prop:disabled"
```

### Agent/Automation Use Cases

```bash
# Single command for perfect context
rdump --format=json "def:Database"

# List all React components
rdump --format=paths "ext:tsx & path:components"

# Find all imports of a module
rdump --format=json "import:react & ext:tsx"
```

### Query Presets (from .rdump.toml)

```toml
# In .rdump.toml
[presets]
rust-src = "ext:rs & !path:tests/"
js-check = "ext:js | ext:jsx"

# With path override
[presets]
rust-src = "ext:rs & path:src/ & !path:tests/"
```

---

## Edge Cases

### Quoted Values

```bash
# Values with spaces
rdump "contains:'fn main()'"
rdump "name:'my file.txt'"
rdump "path:'src/my folder/'"
rdump "contains:'main function'"
rdump "comment:\"HTTP server\""
rdump "contains:\"user's settings\""

# Values with special characters
rdump "matches:'/\\d+/'"
rdump "contains:'foo & bar'"
rdump "contains:'value * 2'"
rdump "matches:'\\(hello world\\)'"
```

### Case Sensitivity

```bash
# ext is case-insensitive
rdump "ext:RS"              # Should match .rs
rdump "ext:Py"              # Should match .py

# contains is case-sensitive
rdump "contains:Main"       # Different from contains:main
rdump "contains:MAIN FUNCTION"

# Operators are case-insensitive
rdump "name:*.log AND contains:error"
rdump "name:*.log and contains:error"
```

### Empty/No Results

```bash
rdump "ext:nonexistent"
rdump "contains:thisshouldnotexist12345"
rdump "path:nonexistent/path"
rdump "def:NonExistent"
rdump "func:non_existent_function"
rdump "struct:NonExistent & ext:go"
rdump "class:NonExistent & ext:java"
rdump "in:docs & func:test_user"
```

### Unicode

```bash
rdump "matches:'こんにちは世界'"
rdump "func:λ"
```

### Invalid Input

```bash
rdump "matches:'('"         # Invalid regex
```

### Operator Precedence

```bash
# NOT binds tightest
rdump "!ext:rs & contains:fn"    # (!ext:rs) & contains:fn

# AND before OR
rdump "ext:rs & contains:fn | ext:py"    # (ext:rs & contains:fn) | ext:py

# Explicit grouping
rdump "ext:rs & (contains:fn | contains:struct)"
```

### Context Lines (Formatter)

```bash
rdump "contains:match"      # With surrounding context
```

---

## Performance Test Cases

### Suboptimal Query Ordering (Tests Optimizer)

```bash
# These should be automatically reordered
rdump "func:main & ext:rs"                              # Should become: ext:rs & func:main
rdump "contains:TODO & size:<1000"                      # Should become: size:<1000 & contains:TODO
rdump "import:react & ext:tsx & path:components"        # Should become: path:components & ext:tsx & import:react
rdump "struct:User & ext:rs"                            # Good: metadata first
rdump "path:models/ & ext:rs & struct:User"             # Best: path first
```

### Large Directory Traversal

```bash
rdump "ext:rs" /large/codebase
rdump "ext:* & size:>0" /very/large/directory
```

### Many Matches

```bash
rdump "ext:rs"                    # All Rust files
rdump "contains:the"              # Common word
rdump "modified:<365d"            # Most files
```

### Wildcard Queries (Full Tree Scan)

```bash
rdump "struct:."
rdump "func:."
rdump "hook:."
rdump "customhook:."
rdump "import:."
```

---

## Benchmark Commands

```bash
# Simple search
hyperfine 'rdump "ext:rs"' --warmup 3

# Content search
hyperfine 'rdump "ext:rs & contains:fn"' --warmup 3

# Code-aware search
hyperfine 'rdump "ext:rs & func:main"' --warmup 3

# Large directory
hyperfine 'rdump "ext:rs" /large/codebase' --warmup 3

# Suboptimal ordering (pre-optimizer)
hyperfine 'rdump "func:main & ext:rs"' --warmup 3

# Optimal ordering (post-optimizer baseline)
hyperfine 'rdump "ext:rs & func:main"' --warmup 3
```

---

## Test Matrix

| Category | Predicate | Basic | Combined | Edge Case |
|----------|-----------|-------|----------|-----------|
| Metadata | ext | `ext:rs` | `ext:rs \| ext:py` | `ext:RS` |
| Metadata | name | `name:*.rs` | `name:*_test.rs & ext:rs` | `name:'my file'` |
| Metadata | path | `path:src` | `path:src & ext:rs` | `path:**/*.rs` |
| Metadata | in | `in:src` | `in:src & struct:User` | `in:.` |
| Metadata | size | `size:>1kb` | `size:>1kb & size:<1mb` | `size:=0` |
| Metadata | modified | `modified:<1d` | `modified:<1d & ext:rs` | `modified:2024-11-19` |
| Content | contains | `contains:fn` | `contains:fn & ext:rs` | `contains:'a & b'` |
| Content | matches | `m:'\\w+'` | `matches:'fn' & ext:rs` | `m:'(?i)test'` |
| Code-aware | def | `def:User` | `def:User & ext:rs` | `def:NonExistent` |
| Code-aware | struct | `struct:User` | `struct:User & path:models` | `struct:.` |
| Code-aware | class | `class:App` | `class:App & ext:java` | - |
| Code-aware | enum | `enum:Status` | - | - |
| Code-aware | interface | `interface:ILog` | `interface:ILog & type:LogLevel` | - |
| Code-aware | trait | `trait:Summary` | - | - |
| Code-aware | type | `type:UserId` | `type:UserId & struct:User` | - |
| Code-aware | impl | `impl:Article` | - | - |
| Code-aware | func | `func:main` | `func:main & ext:py` | `func:.` |
| Code-aware | macro | `macro:my_macro` | - | - |
| Code-aware | import | `import:react` | `import:react & ext:tsx` | `import:.` |
| Code-aware | call | `call:println` | `call:println & ext:rs` | - |
| Code-aware | comment | `comment:TODO` | `struct:Cli & comment:TODO` | `comment:"phrase"` |
| Code-aware | str | `str:error` | `str:error & ext:rs` | `str:/regex/` |
| React | component | `component:App` | `component:App & ext:tsx` | - |
| React | element | `element:div` | `element:Button & prop:disabled` | `element:SVG.Circle` |
| React | hook | `hook:useState` | `hook:useState & !hook:useCallback` | `hook:.` |
| React | customhook | `customhook:useAuth` | - | `customhook:.` |
| React | prop | `prop:onClick` | `element:input & prop:id` | - |
| Logic | AND | `a & b` | `a & b & c` | `a AND b` |
| Logic | OR | `a \| b` | `a \| b \| c` | `a or b` |
| Logic | NOT | `!a` | `!(a & b)` | `not a` |
| Logic | Parens | `(a \| b)` | `(a \| b) & c` | `((a))` |
| Output | format | `--format=json` | `--format=paths -n` | - |

---

## Summary

**Total unique queries: 150+**

Coverage by category:
- Metadata predicates: 30+ queries
- Content predicates: 25+ queries
- Code-aware predicates: 50+ queries
- React-specific: 25+ queries
- Complex boolean logic: 20+ queries
- Edge cases: 15+ queries
