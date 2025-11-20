# rdump Product Requirements Document

**Version:** 1.0
**Status:** Complete
**Last Updated:** 2024
**Author:** Product Team

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [User Personas](#user-personas)
3. [Existing Project Overview](#existing-project-overview)
4. [Use Cases](#use-cases)
5. [Requirements](#requirements)
6. [Technical Constraints and Integration Requirements](#technical-constraints-and-integration-requirements)
7. [Epic and Story Structure](#epic-and-story-structure)
8. [Testing Strategy](#testing-strategy)
9. [Security Considerations](#security-considerations)
10. [Known Limitations](#known-limitations)
11. [Success Metrics](#success-metrics)
12. [Glossary](#glossary)
13. [Appendices](#appendices)

---

## Executive Summary

### Vision Statement

rdump is a next-generation, command-line tool that transforms how developers and AI agents interact with codebases. By combining filesystem metadata filtering, content search, and deep structural code analysis through tree-sitter parsing, rdump provides precise, structured results that eliminate the noise and inefficiency of traditional text-based search tools.

**Core Value Proposition:** rdump is the "ETL for code context" - it Extracts relevant files, Transforms them into structured formats, and Loads them into AI assistants or developer workflows with precision and speed.

**Target Market:** Individual developers, development teams, DevOps engineers, and AI/LLM applications that need to programmatically search and analyze codebases.

### Problem Statement

In the current landscape of AI-assisted development, a significant bottleneck exists in providing Large Language Models (LLMs) and AI agents with relevant context from a codebase. The standard workflow is a painful, manual ritual:

1. Use `ls` to explore the directory structure
2. Use `find` with complex, often-forgotten syntax to locate potentially relevant files
3. Use `grep` or `ripgrep` to see if those files contain specific keywords
4. Finally, use `cat` to dump the contents of the discovered files into a single, massive text blob

This process is slow, error-prone, and produces a low-quality result. The final output is an unstructured firehose of text that is noisy, lacks clear file boundaries, and often exceeds the LLM's context window with irrelevant boilerplate.

#### Quantified Pain Points

| Problem | Impact | Frequency |
|---------|--------|-----------|
| Time spent gathering context | 15-45 minutes per task | Multiple times daily |
| False positives from text search | 50-80% of results irrelevant | Every search |
| Context window overflow | Truncated/missed information | 30% of AI interactions |
| Command syntax recall | Productivity loss, errors | Constant friction |
| Cross-tool coordination | Context switching overhead | Every search task |

#### Root Cause Analysis

The fundamental issue is that traditional Unix tools were designed for a different era and use cases:

- **grep** was designed for line-oriented text processing, not code understanding
- **find** was designed for filesystem traversal, not content analysis
- **cat** was designed for concatenation, not structured output

These tools don't understand:
- Code structure (functions, classes, imports)
- File relationships and dependencies
- The needs of AI consumers (structured data, clear boundaries)
- Modern development patterns (monorepos, polyglot codebases)

### Solution Overview

rdump solves this problem by providing:

- **An Expressive Query Language (RQL):** An intuitive, SQL-like query language that is easy for both humans and AI agents to read, write, and generate
- **High-Performance Search:** Built in Rust with parallel processing via rayon and intelligent file discovery
- **Structural Code Awareness:** Tree-sitter integration enables semantic queries about code structure
- **Structured, Agent-Ready Output:** Multiple output formats (Markdown, JSON) that preserve file boundaries and metadata

#### How rdump Addresses Each Pain Point

| Pain Point | rdump Solution | Result |
|------------|----------------|--------|
| Time gathering context | Single expressive query | 30 seconds vs 30 minutes |
| False positives | Semantic predicates | 90%+ precision |
| Context overflow | Targeted queries | Only relevant code |
| Syntax recall | Intuitive RQL | Easy to learn and remember |
| Tool coordination | All-in-one | No context switching |

#### Technical Approach

rdump achieves its goals through a layered architecture:

1. **Query Layer:** Parse RQL into AST with proper operator precedence
2. **Discovery Layer:** Parallel file walking with intelligent ignore handling
3. **Evaluation Layer:** Short-circuit evaluation with cost-ordered predicates
4. **Semantic Layer:** Tree-sitter parsing for code structure understanding
5. **Output Layer:** Multiple formats with syntax highlighting

### Key Differentiators

| Capability | grep/ripgrep | semgrep | rdump |
|------------|--------------|---------|-------|
| Text search | Excellent | Good | Good |
| Metadata filtering | No | No | Yes |
| Semantic code search | No | Yes | Yes |
| Combined queries | Shell pipes | Limited | Native RQL |
| Output structure | Lines | Patterns | Files + Metadata |
| AI/LLM optimized | No | No | Yes |

#### Detailed Competitive Analysis

**vs ripgrep:**
- ripgrep excels at raw text search speed but cannot filter by metadata (size, date) or understand code structure
- **[TODO: Benchmark needed]** Performance comparison for pure text search vs semantic queries
- ripgrep requires shell pipes for complex queries; rdump handles them natively

> **Action Item:** Run comparative benchmarks on reference codebases (Linux kernel, medium Rust project) to quantify:
> - Text search: `rg "pattern"` vs `rdump "contains:pattern"`
> - Precision: False positive rate for code-specific queries
> - Combined queries: Shell pipeline vs single rdump query

**vs semgrep:**
- semgrep is designed for static analysis rules and linting; rdump is designed for interactive exploration
- semgrep requires YAML configuration; rdump uses inline queries
- semgrep has deeper semantic understanding; rdump has broader metadata support
- Both tools complement each other rather than compete directly

**vs IDE search:**
- IDE search requires the IDE to be open and configured
- IDE search is project-specific; rdump works on any directory
- rdump produces structured output for programmatic use
- rdump is scriptable and CI/CD friendly

### Market Opportunity

The rise of AI-assisted development creates a new category of tooling needs:

- **2023:** 92% of developers use AI coding assistants (GitHub survey)
- **Pain:** Context gathering is the #1 friction point in AI-assisted workflows
- **Gap:** No tool specifically designed for AI context generation
- **Opportunity:** First-mover advantage in "AI-ready developer tools" category

---

## User Personas

### Persona 1: The Senior Developer

**Name:** Sarah Chen
**Role:** Senior Software Engineer
**Experience:** 8 years
**Company:** Mid-size SaaS startup (200 employees)
**Tech Stack:** Rust backend, TypeScript frontend, PostgreSQL

**Goals:**
- Quickly understand unfamiliar codebases
- Efficiently refactor large systems
- Generate context for AI coding assistants
- Mentor junior developers effectively
- Maintain high code quality across the team

**Pain Points:**
- Spends 30+ minutes gathering context for complex refactoring tasks
- grep results are noisy and require manual filtering
- No single tool combines metadata and content search
- Context window limits in AI assistants cause truncation
- Difficult to find all usages before making breaking changes

**Technical Proficiency:**
- Expert in Rust and TypeScript
- Comfortable with command line
- Uses AI assistants (Claude, GitHub Copilot) daily
- Familiar with grep, ripgrep, find, but frustrated by limitations

#### Day in the Life

**9:00 AM - Morning standup**
Sarah learns she needs to refactor the `PaymentProcessor` trait to add async support. This will affect multiple implementations across the codebase.

**9:30 AM - Impact analysis (before rdump)**
- Runs `rg "impl.*PaymentProcessor"` - gets 47 results including comments, tests, and false positives
- Runs `find . -name "*.rs" -exec grep -l "PaymentProcessor" {} \;` - gets file list but no context
- Manually opens each file in IDE to understand the implementation
- Takes 45 minutes to identify 8 actual implementations

**9:30 AM - Impact analysis (with rdump)**
```bash
# Find all implementations
rdump "ext:rs & impl:PaymentProcessor" --format=hunks -C 5

# Find all call sites
rdump "ext:rs & call:PaymentProcessor & !path:test" --format=paths

# Generate context for AI assistant
rdump "ext:rs & (impl:PaymentProcessor | call:process_payment)" --format=markdown > context.md
```
- Identifies all 8 implementations in 30 seconds
- Has structured context ready for AI assistant
- Can see surrounding code for each implementation

**How rdump helps:**
- Single command to find all implementations of a trait/interface
- Combine size, modification time, and code structure in one query
- Structured output perfect for pasting into AI assistants
- Semantic search eliminates false positives from comments and strings

**Example queries:**
```bash
# Find implementations modified recently
rdump "ext:rs & modified:<7d & impl:DatabaseConnection"

# Find large handler functions (candidates for refactoring)
rdump "path:src/api & size:>5kb & func:handle"

# Generate AI context for a specific feature
rdump "path:src/payments & (struct:. | impl:. | func:process)" --format=markdown

# Find all error handling patterns
rdump "ext:rs & (contains:anyhow | contains:thiserror) & func:."
```

**Value Delivered:**
- 90% reduction in context gathering time
- Higher quality AI interactions due to precise context
- Confidence in refactoring due to complete impact analysis
- Better code reviews with ability to quickly find patterns

---

### Persona 2: The AI Agent

**Name:** CodeBot
**Role:** Autonomous Coding Agent
**Experience:** N/A (LLM-based)
**Platform:** Claude, GPT-4, or similar LLM with tool use capabilities
**Integration:** MCP server, function calling, or shell execution

**Goals:**
- Acquire perfect context in minimal steps
- Understand code structure without reading every file
- Generate structured output for further processing
- Minimize token usage while maximizing relevance
- Provide deterministic, reproducible results

**Pain Points:**
- Shell command chaining is brittle and error-prone
- Text search returns too many false positives
- Cannot easily determine file structure
- Multiple tool calls consume tokens and time
- Unstructured output is hard to parse programmatically

**Technical Requirements:**
- Deterministic output for consistent behavior
- Structured formats (JSON) for easy parsing
- Clear error messages for self-correction
- Bounded output size to fit context windows
- Fast execution for responsive interactions

#### Agent Workflow Analysis

**Traditional Approach (without rdump):**
```
Step 1: ls -la src/                    # 1 tool call, ~50 tokens output
Step 2: find src -name "*.rs"          # 1 tool call, ~200 tokens output
Step 3: grep -l "UserService" src/*.rs # 1 tool call, ~100 tokens output
Step 4: cat src/user.rs                # 1 tool call, ~500 tokens output
Step 5: grep -n "impl" src/user.rs     # 1 tool call, ~50 tokens output

Total: 5 tool calls, ~900 tokens, brittle pipeline
```

**rdump Approach:**
```
Step 1: rdump "ext:rs & impl:UserService" --format=json

Total: 1 tool call, ~600 tokens, precise results
```

**Efficiency Gains:**
- 80% reduction in tool calls
- 30% reduction in token usage
- 100% elimination of false positives
- Zero pipeline failures

#### Typical Agent Tasks

**Task 1: Understand a Feature**
```bash
# Get all code related to authentication
rdump "path:src/auth & (func:. | struct:. | impl:.)" --format=json
```

**Task 2: Find Implementation Details**
```bash
# Find how errors are handled
rdump "ext:rs & (contains:anyhow | contains:thiserror)" --format=hunks -C 3
```

**Task 3: Locate Test Coverage**
```bash
# Find tests for a specific module
rdump "path:tests & contains:user_service" --format=paths
```

**Task 4: Analyze Dependencies**
```bash
# Find all imports of a library
rdump "import:serde" --format=json
```

#### Integration Patterns

**MCP Server Integration:**
```json
{
  "name": "rdump",
  "description": "Search codebase with semantic understanding",
  "parameters": {
    "query": "RQL query string",
    "format": "json|markdown|paths|hunks",
    "context_lines": "number of context lines for hunks"
  }
}
```

**Function Calling Schema:**
```json
{
  "name": "search_codebase",
  "description": "Search for code patterns using rdump",
  "parameters": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "RQL query (e.g., 'ext:rs & func:main')"
      },
      "format": {
        "type": "string",
        "enum": ["json", "markdown", "paths", "hunks"],
        "default": "json"
      }
    },
    "required": ["query"]
  }
}
```

**How rdump helps:**
- Single tool replaces ls + find + grep + cat workflow
- Semantic predicates eliminate false positives
- JSON output is directly parseable
- Deterministic results enable reliable agent behavior
- Bounded output prevents context overflow

**Example queries:**
```bash
# Structured output for parsing
rdump --format=json "ext:rs & def:User"

# Find React patterns
rdump --format=json "ext:tsx & import:react & hook:useState"

# Get specific function implementations
rdump --format=json "ext:rs & func:process_payment"

# Find all API endpoints
rdump --format=json "path:api/ & func:handle"
```

**Value for AI Agents:**
- **Precision:** 95%+ relevant results vs 20-50% with grep
- **Efficiency:** 5x fewer tool calls
- **Reliability:** No brittle shell pipelines
- **Parsability:** Native JSON output
- **Consistency:** Deterministic results

---

### Persona 3: The DevOps Engineer

**Name:** Marcus Johnson
**Role:** DevOps/SRE
**Experience:** 5 years

**Goals:**
- Audit configuration files across projects
- Find security issues in infrastructure code
- Automate codebase analysis in CI/CD

**Pain Points:**
- Configuration files scattered across many formats
- Manual auditing is tedious and error-prone
- Difficult to integrate custom checks into pipelines

**How rdump helps:**
- Filter by file type and content in one query
- Pipe output to other tools for automated processing
- Consistent, scriptable interface

**Example queries:**
```bash
rdump "ext:yaml & !path:test & contains:password"
rdump "(ext:toml | ext:yaml) & modified:<1d" --format=paths
```

---

### Persona 4: The Junior Developer

**Name:** Alex Rivera
**Role:** Junior Developer
**Experience:** 1 year

**Goals:**
- Learn new codebase quickly
- Find examples of patterns to follow
- Understand how existing features work

**Pain Points:**
- Overwhelmed by large codebases
- Doesn't know where to start looking
- grep results require too much context to understand

**How rdump helps:**
- Find all examples of a pattern with semantic search
- Markdown output with syntax highlighting is readable
- Combine with context lines to see surrounding code

**Example queries:**
```bash
rdump "ext:py & func:test_" --format=hunks -C 5
rdump "component:Button & prop:onClick"
```

---

### Persona 5: The QA Engineer

**Name:** Jordan Taylor
**Role:** QA Engineer / SDET
**Experience:** 4 years

**Goals:**
- Ensure comprehensive test coverage
- Find untested code paths
- Verify test naming conventions
- Identify flaky or skipped tests

**Pain Points:**
- Difficult to correlate tests with implementation
- Hard to find all tests for a specific feature
- Manual verification of test coverage is tedious
- No easy way to audit test quality

**How rdump helps:**
- Find all tests for a specific function or module
- Identify code without corresponding tests
- Audit test naming and organization
- Find skipped or disabled tests

**Example queries:**
```bash
# Find all tests for a specific module
rdump "path:tests/ & (contains:UserService | name:*user*)"

# Find skipped tests
rdump "ext:py & (contains:@pytest.mark.skip | contains:@unittest.skip)"

# Find tests without assertions
rdump "ext:rs & func:test_ & !contains:assert"
```

---

### Persona 6: The Tech Lead

**Name:** David Kim
**Role:** Technical Lead / Architect
**Experience:** 12 years

**Goals:**
- Maintain architectural consistency
- Review code quality at scale
- Onboard new team members efficiently
- Make informed technical decisions

**Pain Points:**
- Hard to enforce architectural boundaries
- Code review doesn't scale
- Documentation gets out of sync
- Difficult to assess technical debt

**How rdump helps:**
- Audit architectural patterns across codebase
- Find violations of coding standards
- Generate codebase reports for decision-making
- Create onboarding materials from actual code

**Example queries:**
```bash
# Find potential architectural violations (direct DB access outside repository layer)
rdump "!path:repository/ & !path:db/ & (import:sqlx | import:diesel | contains:SELECT)"

# Find large files that may need refactoring
rdump "(ext:rs | ext:ts | ext:py) & size:>100kb" --format=find

# Find circular dependency candidates
rdump "path:moduleA/ & import:moduleB" && rdump "path:moduleB/ & import:moduleA"

# Audit error handling patterns
rdump "ext:rs & (contains:unwrap() | contains:expect(\")" --format=hunks -C 2

# Find deprecated patterns
rdump "contains:@deprecated | contains:#[deprecated] | comment:DEPRECATED"
```

---

### Persona 7: The Open Source Contributor

**Name:** Priya Sharma
**Role:** Open Source Contributor
**Experience:** 3 years (part-time)

**Goals:**
- Quickly understand new codebases
- Find "good first issues" to work on
- Follow existing patterns and conventions
- Make meaningful contributions efficiently

**Pain Points:**
- Limited time to learn large codebases
- Hard to find where to make changes
- Existing tools require project-specific setup
- Documentation often outdated

**How rdump helps:**
- Zero-config exploration of any project
- Find examples of patterns to follow
- Locate TODO/FIXME for contribution opportunities
- Understand code without reading everything

**Example queries:**
```bash
# Find contribution opportunities
rdump "comment:TODO | comment:FIXME | comment:HACK | comment:XXX"

# Find examples of similar features
rdump "path:handlers/ & func:create_" --format=hunks -C 10

# Understand module structure
rdump "ext:rs & (func:new | func:default)" --format=paths

# Find documentation that needs updating
rdump "comment:OUTDATED | comment:UPDATE | comment:REVIEW"
```

---

## Existing Project Overview

### Analysis Source

- IDE-based fresh analysis
- Existing documentation: `README.md`
- Source code analysis of `rdump/` directory

### Current Project State

**rdump v0.1.7** is a next-generation, command-line tool for developers that finds and processes files by combining filesystem metadata, content matching, and deep structural code analysis. Built in Rust for blazing-fast performance, it goes beyond text-based search tools like `grep` and `ripgrep` by using tree-sitter to parse code into syntax trees, enabling semantic queries about code structure.

The tool bridges the "context gap" between developers/AI agents and codebases by providing a single, powerful interface to extract, transform, and load code context for language models.

### Project Statistics

| Metric | Value |
|--------|-------|
| Version | 0.1.7 |
| Language | Rust (Edition 2021) |
| Dependencies | 25+ crates |
| Supported Languages | 6 (Rust, Python, JavaScript, TypeScript, Go, Java) |
| Output Formats | 6 (markdown, json, cat, paths, hunks, find) |
| Predicate Types | 30+ |

### Available Documentation

| Documentation | Status | Location |
|--------------|--------|----------|
| User Guide (README) | Complete | `README.md` |
| Performance Documentation | Partial | `docs/performance-optimizations.md` |
| API Documentation | Complete | Inline + README |
| Coding Standards | Partial | Implicit in codebase |
| Technical Debt Documentation | Not documented | N/A |

### Enhancement Scope Definition

**Enhancement Type:** Documentation of Existing Complete Project

**Description:** This PRD documents the complete rdump v0.1.7 tool - a code-aware file search utility with a custom query language (RQL), multiple output formats, parallel execution, and semantic code search capabilities using tree-sitter.

**Impact Assessment:** N/A - documenting existing, stable functionality

### Goals

**Primary Goals:**
- Provide a fast, expressive file search tool with zero configuration
- Enable complex queries through the RQL (rdump Query Language)
- Support structural/semantic code queries using tree-sitter
- Achieve high performance through parallel execution with rayon

**Secondary Goals:**
- Ensure deterministic output for reproducibility
- Provide multiple output formats optimized for humans and AI agents
- Support multiple programming languages for code-aware search
- Enable easy extension to new languages

**Non-Goals:**
- Full IDE integration (VS Code extension, etc.)
- Real-time file watching / incremental search
- Cross-repository search
- Source code modification capabilities

### Background Context

rdump was created to solve the "Developer-to-LLM Context Gap" - the painful, manual process of using multiple shell commands (`ls`, `find`, `grep`, `cat`) to gather relevant code context for AI-assisted development. Instead of an unstructured firehose of text, rdump provides structured, agent-ready output through an intuitive query language.

The tool serves both human developers for rapid context grabbing and codebase exploration, and AI agents as their primary file interaction tool - enabling perfect context acquisition in a single step rather than dozens of brittle shell commands.

#### Historical Context

The project evolved through several phases:

1. **Phase 1.0 (Core):** Basic query language with metadata and content predicates
2. **Phase 1.1 (Performance):** Parallel processing with rayon, short-circuit evaluation
3. **Phase 2.0 (Code-Aware):** Tree-sitter integration for Rust semantic predicates
4. **Phase 2.1 (Multi-Language):** Extended to Python, JavaScript, TypeScript, Go, Java
5. **Phase 2.2 (React):** Specialized predicates for React component analysis

---

## Use Cases

### Use Case 1: Rapid Context Generation for AI Assistants

**Actor:** Senior Developer
**Goal:** Generate focused context for an AI coding assistant
**Preconditions:**
- Working on a feature that requires understanding existing patterns
- AI assistant (Claude, ChatGPT, GitHub Copilot) is available
- Codebase is accessible on local filesystem

**Trigger:** Developer needs to understand existing code patterns before implementing a new feature or making changes.

#### Main Flow

| Step | Actor | Action | System Response |
|------|-------|--------|-----------------|
| 1 | Developer | Identifies the pattern/feature area to understand | N/A |
| 2 | Developer | Constructs rdump query with path, extension, and semantic predicates | N/A |
| 3 | System | Parses query into AST | Validates syntax, reports errors |
| 4 | System | Discovers candidate files respecting ignore patterns | Parallel directory traversal |
| 5 | System | Evaluates predicates with short-circuit optimization | Filters to matching files |
| 6 | System | Formats output according to specified format | Structured Markdown/JSON |
| 7 | Developer | Reviews output for relevance | N/A |
| 8 | Developer | Pastes output into AI assistant | N/A |
| 9 | AI Assistant | Analyzes context and provides insights | N/A |

#### Alternative Flows

**A1: Query returns too many results**
- At step 7, developer sees output is too large for context window
- Developer adds more specific predicates (path, size, modified)
- Returns to step 3

**A2: Query returns no results**
- At step 6, system returns empty results
- Developer uses `--verbose` to debug
- Developer adjusts predicates (check ignore files, file extensions)
- Returns to step 3

**A3: Need to iterate on context**
- At step 9, AI assistant requests more specific information
- Developer constructs follow-up query based on AI feedback
- Returns to step 2

#### Example Session

```bash
# Initial broad query - too many results
rdump "ext:rs & func:." --format=count
# Output: 347 files

# Narrow by path
rdump "path:src/api & ext:rs & func:." --format=count
# Output: 23 files

# Add semantic filter for handlers
rdump "path:src/api & ext:rs & func:handle" --format=markdown
# Output: 8 files with handler implementations

# Generate final context with context lines
rdump "path:src/api & ext:rs & func:handle" --format=markdown > context.md
# Paste context.md contents into AI assistant
```

**Example queries:**
```bash
# Find all API handler functions in Go, excluding tests
rdump "path:api/ & ext:go & func:Handle & !path:test" --format=markdown

# Find all database models with their relationships
rdump "path:models/ & (struct:. | class:.) & !name:*_test*" --format=json

# Get authentication-related code for security review
rdump "path:src/auth & (func:. | struct:. | impl:.)" --format=markdown

# Find all uses of a specific pattern
rdump "ext:rs & contains:async fn & contains:Result<" --format=hunks -C 3
```

#### Success Criteria

| Criterion | Metric | Target |
|-----------|--------|--------|
| Query precision | % of results that are relevant | > 90% |
| Time to context | From query to usable output | < 5 seconds |
| Context fit | Output fits in AI context window | 95% of queries |
| Iteration speed | Time to refine query | < 30 seconds |

#### Failure Modes

| Failure | Detection | Recovery |
|---------|-----------|----------|
| Query syntax error | Parser error message | Fix syntax per error guidance |
| No results | Empty output | Use --verbose, check predicates |
| Too many results | Output size warning | Add more filters |
| Wrong file type | Unexpected content | Check ext: predicate |
| Missed files | Manual verification | Check ignore files with --no-ignore |

#### Business Value

- **Time savings:** 45 minutes → 5 minutes per context gathering task
- **Quality improvement:** More precise context → better AI responses
- **Developer satisfaction:** Reduced frustration with manual searching
- **Knowledge transfer:** Easier to understand unfamiliar code areas

---

### Use Case 2: Codebase Security Audit

**Actor:** DevOps Engineer
**Goal:** Find potential security issues across codebase
**Preconditions:** Need to audit before deployment or compliance review

**Main Flow:**
1. Engineer runs series of security-focused queries
2. Queries target hardcoded secrets, unsafe patterns, deprecated APIs
3. Results are exported for review or integrated into CI pipeline
4. Issues are triaged and assigned for remediation

**Example:**
```bash
# Find potential hardcoded secrets
rdump "!path:test & !path:mock & str:/[A-Za-z0-9_\\-]{20,}/"

# Find SQL injection vulnerabilities
rdump "str:/SELECT.*\\+.*\\+/ | str:/execute.*\\+/"

# Find disabled security features
rdump "comment:nosec | comment:nolint | comment:skipcq"

# Find uses of deprecated crypto
rdump "import:md5 | import:sha1 | call:md5 | call:sha1"
```

**Success Criteria:**
- Queries identify real security issues
- False positive rate < 20%
- Can be automated in CI/CD pipeline

---

### Use Case 3: Large-Scale Refactoring

**Actor:** Senior Developer
**Goal:** Safely rename or modify a widely-used component
**Preconditions:**
- Component is used across multiple modules
- Breaking changes could cause runtime errors
- Comprehensive test coverage exists
- Version control is available for rollback

**Trigger:** Need to rename `UserService` to `AccountService` or add async support to all methods.

#### Main Flow

| Step | Actor | Action | System Response | Duration |
|------|-------|--------|-----------------|----------|
| 1 | Developer | Identify component to refactor | N/A | N/A |
| 2 | Developer | Query for all definitions | Returns all struct/trait/impl definitions | 2s |
| 3 | Developer | Query for all imports | Returns all files importing component | 1s |
| 4 | Developer | Query for all call sites | Returns all function/method calls | 3s |
| 5 | Developer | Export results to checklist | Creates refactoring plan | N/A |
| 6 | Developer | Execute refactoring | N/A | Varies |
| 7 | Developer | Verify no remaining usages | Returns 0 results | 1s |

#### Detailed Refactoring Workflow

**Phase 1: Discovery**

```bash
# Step 2: Find all definitions
rdump "ext:rs & (struct:UserService | trait:UserService | impl:UserService)" \
  --format=hunks -C 10 > definitions.txt

# Review: 3 files found
# - src/services/user.rs (struct + impl)
# - src/traits/service.rs (trait)
# - src/services/mock.rs (mock impl)

# Step 3: Find all imports
rdump "ext:rs & import:UserService" --format=paths > imports.txt

# Review: 12 files import UserService
wc -l imports.txt
# 12 imports.txt

# Step 4: Find all call sites
rdump "ext:rs & call:UserService" --format=hunks -C 3 > usages.txt

# Review: 28 call sites found
grep -c "^---" usages.txt
# 28
```

**Phase 2: Impact Analysis**

```bash
# Find tests that need updating
rdump "path:tests & contains:UserService" --format=paths > tests.txt

# Find documentation references
rdump "(ext:md | ext:rs) & contains:UserService & comment:." --format=paths > docs.txt

# Find configuration files
rdump "(ext:toml | ext:yaml | ext:json) & contains:UserService" --format=paths > config.txt

# Generate complete impact report
cat <<EOF > refactoring-plan.md
# UserService → AccountService Refactoring Plan

## Files to Modify

### Definitions (3 files)
$(cat definitions.txt | grep "^---" | cut -d' ' -f2)

### Imports (12 files)
$(cat imports.txt)

### Call Sites (28 locations)
$(grep "^---" usages.txt | cut -d' ' -f2 | sort -u)

### Tests (8 files)
$(cat tests.txt)

### Documentation (4 files)
$(cat docs.txt)

## Execution Order
1. Update trait definition
2. Update struct definition
3. Update implementations
4. Update imports (automated with sed)
5. Update call sites
6. Update tests
7. Update documentation

## Rollback Plan
- Git branch: feature/user-to-account-rename
- Revert commit if tests fail
EOF
```

**Phase 3: Execution**

```bash
# Automated rename for simple cases
rdump "ext:rs & contains:UserService" --format=paths | \
  xargs sed -i 's/UserService/AccountService/g'

# Verify no remaining references
rdump "ext:rs & contains:UserService" --format=count
# 0

# Run tests
cargo test

# If tests pass, commit
git add -A
git commit -m "Rename UserService to AccountService"
```

**Phase 4: Verification**

```bash
# Verify new name is used correctly
rdump "ext:rs & (struct:AccountService | impl:AccountService)" --format=count
# 3 (same as before)

rdump "ext:rs & import:AccountService" --format=count
# 12 (same as before)

# Verify no orphaned references
rdump "ext:rs & contains:UserService" --format=count
# 0
```

#### Alternative Flows

**A1: Partial Refactoring (Deprecation)**
- At step 6, developer decides to keep old name with deprecation
- Creates type alias: `type UserService = AccountService;`
- Adds `#[deprecated]` attribute
- Gradually migrate callers

```bash
# Find files still using deprecated name
rdump "ext:rs & import:UserService & !path:deprecated" --format=paths
```

**A2: Async Migration**
- Instead of rename, adding `async` to all methods
- Need to find all call sites and add `.await`

```bash
# Find sync calls that need .await
rdump "ext:rs & call:user_service & !contains:.await" --format=hunks -C 2
```

**A3: Breaking API Change**
- Adding required parameter to function
- Need to update all call sites with new argument

```bash
# Find all calls to update
rdump "ext:rs & call:get_user" --format=hunks -C 3

# After update, verify signature matches
rdump "ext:rs & call:get_user & !contains:get_user(" --format=paths
# Should be empty if all updated correctly
```

#### Example Queries

```bash
# Find the definition
rdump "def:UserService" --format=hunks -C 5

# Find all imports
rdump "import:UserService" --format=paths

# Find all usages (call sites)
rdump "call:UserService" --format=hunks -C 3

# Find related test files
rdump "name:*user*service*test* | (path:test & contains:UserService)"

# Find usages excluding tests
rdump "!path:test & !path:mock & call:UserService" --format=paths

# Find direct instantiations
rdump "ext:rs & contains:UserService::new" --format=hunks -C 2

# Find method calls
rdump "ext:rs & contains:.get_user(" --format=hunks
```

#### Success Criteria

| Criterion | Metric | Target |
|-----------|--------|--------|
| Definition coverage | % of definitions found | 100% |
| Import coverage | % of imports found | 100% |
| Call site coverage | % of call sites found | 100% |
| Refactoring completeness | Remaining old references | 0 |
| Test pass rate | Tests passing after refactor | 100% |
| Runtime errors | Errors in staging/prod | 0 |

#### Risk Mitigation

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Missed dynamic calls | Low | Search for string literals with component name |
| Reflection-based usages | Low | Search for type name in strings |
| External dependencies | Medium | Check API consumers separately |
| Database migration needed | Medium | Check ORM models and migrations |

#### Time Estimates

| Task | Without rdump | With rdump | Savings |
|------|---------------|------------|---------|
| Find all definitions | 30 min | 2 min | 93% |
| Find all imports | 20 min | 1 min | 95% |
| Find all call sites | 60 min | 3 min | 95% |
| Generate impact report | 45 min | 5 min | 89% |
| Verify completion | 30 min | 2 min | 93% |
| **Total** | **3 hours** | **13 min** | **93%** |

---

### Use Case 4: React Component Analysis

**Actor:** Frontend Developer
**Goal:** Analyze React component patterns and optimize performance
**Preconditions:** Working on React/TypeScript codebase

**Main Flow:**
1. Developer identifies components that may have performance issues
2. Developer queries for specific hook usage patterns
3. Developer identifies components missing optimizations
4. Developer applies fixes based on analysis

**Example:**
```bash
# Find components with useState but no useMemo/useCallback
rdump "ext:tsx & hook:useState & !(hook:useMemo | hook:useCallback)"

# Find components not wrapped in React.memo
rdump "component:. & !contains:React.memo" --format=paths

# Find all custom hooks
rdump "customhook:." --format=hunks

# Find components using specific props
rdump "element:Button & prop:disabled" -C 2

# Find large components that might need splitting
rdump "ext:tsx & size:>10kb & component:."
```

**Success Criteria:**
- Performance anti-patterns identified
- Clear list of components to optimize
- Measurable performance improvement after fixes

---

### Use Case 5: Onboarding to New Codebase

**Actor:** Junior Developer
**Goal:** Understand codebase structure and patterns quickly
**Preconditions:** Just joined team, unfamiliar with codebase

**Main Flow:**
1. Developer explores high-level structure by file types
2. Developer identifies main entry points and patterns
3. Developer finds examples of specific patterns to follow
4. Developer understands testing conventions

**Example:**
```bash
# Get overview of file types
rdump "ext:rs" --format=find | head -20
rdump "ext:py" --format=find | head -20

# Find main entry points
rdump "func:main | func:__main__" --format=hunks

# Find all public API endpoints
rdump "path:api/ & (func:get | func:post | func:put | func:delete)"

# Find test examples to follow
rdump "ext:rs & name:*test* & func:test_" --format=hunks -C 3

# Find documentation comments
rdump "comment:TODO | comment:FIXME | comment:NOTE"
```

**Success Criteria:**
- Developer understands codebase structure within 1 day
- Developer can find relevant examples for any task
- Reduced time asking teammates for help

---

### Use Case 6: CI/CD Integration

**Actor:** DevOps Engineer
**Goal:** Automate code quality checks in pipeline
**Preconditions:** Setting up or improving CI/CD pipeline

**Main Flow:**
1. Engineer defines quality gates as rdump queries
2. Queries are integrated into CI pipeline
3. Pipeline fails if queries return results (issues found)
4. Results are reported in PR comments

**Example:**
```bash
# Check for console.log in production code
rdump "ext:ts & !path:test & call:console.log" --format=count

# Check for TODO comments in critical paths
rdump "path:src/core & comment:TODO" --format=paths

# Ensure all API endpoints have error handling
rdump "path:api/ & !contains:try & func:handle"

# Check for large files that need review
rdump "(ext:ts | ext:js) & size:>50kb" --format=find
```

**Success Criteria:**
- Quality gates prevent issues from merging
- Clear, actionable output in CI logs
- Fast execution (< 30 seconds for full check)

---

## Requirements

### Functional Requirements

#### Core Query Language

- **FR1:** The tool shall accept a query string using the rdump Query Language (RQL) as the primary input mechanism
  - FR1.1: Query string shall be passed as a positional argument
  - FR1.2: Query string should be quoted to prevent shell interpretation
  - FR1.3: Query string shall be validated before execution

  **Rationale:** RQL is the primary interface for users. Without a query language, users would need to use multiple flags and options, reducing expressiveness and increasing complexity.
  **Dependencies:** None (foundational)
  **Acceptance Test:** `rdump "ext:rs"` returns all Rust files

- **FR2:** RQL shall support boolean operators: `&` (AND), `|` (OR), `!` (NOT) with standard precedence
  - FR2.1: `!` (NOT) has highest precedence
  - FR2.2: `&` (AND) has higher precedence than `|` (OR)
  - FR2.3: Operators can also be written as `and`, `or`, `not`

  **Rationale:** Boolean operators enable composition of simple predicates into powerful queries. Standard precedence matches user expectations from programming languages and reduces need for parentheses.
  **Dependencies:** FR1
  **Acceptance Test:** `rdump "ext:rs & contains:fn | ext:py"` correctly parses as `(ext:rs & contains:fn) | ext:py`

  **Design Decisions:**
  - Chose `&` and `|` over `AND`/`OR` for brevity in CLI context
  - Support both symbol and word forms for flexibility
  - Standard precedence (NOT > AND > OR) matches most programming languages

- **FR3:** RQL shall support grouping with parentheses for explicit precedence control
  - FR3.1: Nested parentheses shall be supported to any depth
  - FR3.2: Unmatched parentheses shall produce clear error messages

  **Rationale:** Parentheses allow users to override default precedence when needed, essential for complex queries.
  **Dependencies:** FR1, FR2
  **Acceptance Test:** `rdump "(ext:rs | ext:py) & contains:main"` correctly groups the OR before the AND

  **Error Handling:**
  - Unmatched `(`: "Error: Unclosed parenthesis at position N"
  - Unmatched `)`: "Error: Unexpected closing parenthesis at position N"
  - Empty `()`: "Error: Empty group at position N"

- **FR4:** RQL shall support quoting for values containing special characters
  - FR4.1: Single quotes `'...'` shall be supported
  - FR4.2: Double quotes `"..."` shall be supported
  - FR4.3: Unquoted values shall be supported for simple strings

  **Rationale:** Many search patterns contain characters that are special in RQL (spaces, parentheses, operators). Quoting allows these to be used literally.
  **Dependencies:** FR1
  **Acceptance Test:** `rdump "contains:'fn main()'"` finds literal string "fn main()"

  **Examples:**
  - `contains:'hello world'` - space in value
  - `matches:"/^fn\s+\w+/"` - regex with special chars
  - `name:"file (1).txt"` - parentheses in filename

#### Metadata Predicates

- **FR5:** The tool shall support the `ext:` predicate for file extension matching
  - FR5.1: Extension matching shall be case-insensitive
  - FR5.2: Extension shall not include the leading dot
  - FR5.3: Example: `ext:rs` matches `file.rs` and `file.RS`

- **FR6:** The tool shall support the `name:` predicate for filename matching
  - FR6.1: Name matching shall support glob patterns
  - FR6.2: Name shall match against basename only (not full path)
  - FR6.3: Example: `name:"*_test.rs"` matches `user_test.rs`

- **FR7:** The tool shall support the `path:` predicate for path substring matching
  - FR7.1: Path matching shall check full canonical path
  - FR7.2: Match shall succeed if substring appears anywhere in path
  - FR7.3: Glob patterns shall be supported
  - FR7.4: Example: `path:src/api` matches `/home/user/project/src/api/handler.rs`

- **FR8:** The tool shall support the `path_exact:` predicate for exact path matching
  - FR8.1: Match shall require exact canonical path match
  - FR8.2: Example: `path_exact:/home/user/project/main.rs`

- **FR9:** The tool shall support the `size:` predicate for file size filtering
  - FR9.1: Operators `>`, `<`, `=`, `>=`, `<=` shall be supported
  - FR9.2: Units `b`, `kb`, `mb`, `gb` shall be supported
  - FR9.3: No space between operator and number
  - FR9.4: Example: `size:>100kb`, `size:<=10mb`

- **FR10:** The tool shall support the `modified:` predicate for modification time filtering
  - FR10.1: Operators `>`, `<`, `=` shall be supported
  - FR10.2: Units `s` (seconds), `m` (minutes), `h` (hours), `d` (days), `w` (weeks) shall be supported
  - FR10.3: `<` means "within the last" (recent), `>` means "older than"
  - FR10.4: Example: `modified:<2d` matches files modified in last 48 hours

- **FR11:** The tool shall support the `in:` predicate for directory membership
  - FR11.1: Exact directory match without glob
  - FR11.2: Recursive match with glob patterns
  - FR11.3: Example: `in:"src/api"` matches files directly in that directory

#### Content Predicates

- **FR12:** The tool shall support the `contains:` (alias `c:`) predicate for literal substring search
  - FR12.1: Search shall be case-sensitive by default
  - FR12.2: Search shall find exact substring matches
  - FR12.3: Content shall only be read if metadata predicates pass
  - FR12.4: Example: `contains:'fn main()'`

- **FR13:** The tool shall support the `matches:` (alias `m:`) predicate for regex search
  - FR13.1: Regex syntax shall follow Rust regex crate conventions
  - FR13.2: Search shall be case-sensitive by default
  - FR13.3: Example: `matches:'/struct \w+/'`

#### Code-Aware Semantic Predicates

- **FR14:** The tool shall support generic semantic predicates across all languages
  - FR14.1: `def:` finds generic definitions (class, struct, trait, etc.)
  - FR14.2: `func:` finds function or method definitions
  - FR14.3: `import:` finds import/use/require statements
  - FR14.4: `call:` finds function or method call sites
  - FR14.5: `comment:` finds text within any comment node
  - FR14.6: `str:` finds text within any string literal node

- **FR15:** The tool shall support Rust-specific semantic predicates
  - FR15.1: `struct:` finds struct definitions
  - FR15.2: `enum:` finds enum definitions
  - FR15.3: `trait:` finds trait definitions
  - FR15.4: `impl:` finds impl blocks
  - FR15.5: `type:` finds type aliases
  - FR15.6: `macro:` finds macro definitions

- **FR16:** The tool shall support Python-specific semantic predicates
  - FR16.1: `class:` finds class definitions
  - FR16.2: `func:` finds function definitions including methods

- **FR17:** The tool shall support JavaScript/TypeScript-specific semantic predicates
  - FR17.1: `class:` finds class definitions
  - FR17.2: `interface:` finds interface definitions (TypeScript)
  - FR17.3: `enum:` finds enum definitions (TypeScript)
  - FR17.4: `type:` finds type aliases (TypeScript)

- **FR18:** The tool shall support Go-specific semantic predicates
  - FR18.1: `struct:` finds struct definitions
  - FR18.2: `interface:` finds interface definitions
  - FR18.3: `type:` finds type definitions

- **FR19:** The tool shall support Java-specific semantic predicates
  - FR19.1: `class:` finds class definitions
  - FR19.2: `interface:` finds interface definitions
  - FR19.3: `enum:` finds enum definitions

- **FR20:** The tool shall support the "match any" wildcard `.` for semantic predicates
  - FR20.1: Example: `func:.` matches any function definition
  - FR20.2: Example: `!import:.` matches files with no imports

#### React-Specific Predicates

- **FR21:** The tool shall support React-specific predicates for JSX/TSX files
  - FR21.1: `component:` finds React component definitions
  - FR21.2: `element:` finds JSX element tags
  - FR21.3: `hook:` finds hook calls (functions starting with `use`)
  - FR21.4: `customhook:` finds custom hook definitions
  - FR21.5: `prop:` finds props passed to JSX elements

#### Output Formats

- **FR22:** The tool shall support multiple output formats via `--format` option
  - FR22.1: `markdown` - Headers, metadata, and fenced code blocks
  - FR22.2: `json` - Structured data with path, size, modified, content
  - FR22.3: `cat` - Concatenated file contents
  - FR22.4: `paths` - Newline-separated file paths only
  - FR22.5: `hunks` - Matching code blocks with context
  - FR22.6: `find` - ls -l style output with metadata

- **FR23:** The tool shall support line number display
  - FR23.1: `--line-numbers` or `-n` flag enables line numbers
  - FR23.2: Line numbers prepended in `markdown`, `cat`, `hunks` formats
  - FR23.3: JSON content field always contains original unmodified content

- **FR24:** The tool shall support context lines for hunks format
  - FR24.1: `-C <N>` shows N lines before and after matches
  - FR24.2: `-B <N>` shows N lines before matches
  - FR24.3: `-A <N>` shows N lines after matches

- **FR25:** The tool shall support output redirection
  - FR25.1: `--output` or `-o` writes to specified file
  - FR25.2: Default is stdout

- **FR26:** The tool shall support syntax highlighting
  - FR26.1: Highlighting via syntect library
  - FR26.2: `--color` flag controls highlighting (always/never/auto)
  - FR26.3: Auto-detection based on terminal capabilities

#### Ignore File Support

- **FR27:** The tool shall respect ignore file patterns by default
  - FR27.1: `.gitignore` patterns are respected
  - FR27.2: `.rdumpignore` patterns are respected with highest precedence
  - FR27.3: Global gitignore is respected
  - FR27.4: Built-in ignores for common directories (node_modules, target, etc.)

- **FR28:** The tool shall provide flags to control ignore behavior
  - FR28.1: `--no-ignore` disables all ignore file logic
  - FR28.2: `--hidden` includes hidden files/directories

#### Configuration and Presets

- **FR29:** The tool shall support configuration files
  - FR29.1: Global config at `~/.config/rdump/config.toml`
  - FR29.2: Local config at `.rdump.toml` overrides global
  - FR29.3: TOML format for configuration

- **FR30:** The tool shall support query presets
  - FR30.1: `rdump preset list` shows all presets
  - FR30.2: `rdump preset add <name> <query>` creates/updates preset
  - FR30.3: `rdump preset remove <name>` deletes preset
  - FR30.4: `--preset` or `-p` flag uses saved preset
  - FR30.5: Multiple presets can be combined

#### Introspection Commands

- **FR31:** The tool shall provide language introspection
  - FR31.1: `rdump lang list` shows supported languages and extensions
  - FR31.2: `rdump lang describe <language>` shows available predicates
  - FR31.3: Output includes metadata, content, and semantic predicates

- **FR32:** The tool shall provide standard CLI information
  - FR32.1: `--help` or `-h` displays help information
  - FR32.2: `--version` or `-V` displays version information
  - FR32.3: `--verbose` or `-v` enables debug output

---

### Non-Functional Requirements

#### Performance

- **NFR1:** The tool shall be written in Rust (Edition 2021) for memory safety and performance
  - NFR1.1: No runtime garbage collection overhead
  - NFR1.2: Zero-cost abstractions where possible
  - NFR1.3: Memory-safe by construction

- **NFR2:** The tool shall use parallel execution via rayon for multi-core utilization
  - NFR2.1: File processing distributed across all available cores
  - NFR2.2: `--threads` option to control thread count
  - NFR2.3: Automatic thread pool management

- **NFR3:** The tool shall perform short-circuit evaluation to skip expensive operations
  - NFR3.1: In `A & B`, if `A` is false, `B` is not evaluated
  - NFR3.2: In `A | B`, if `A` is true, `B` is not evaluated
  - NFR3.3: Significant performance improvement for complex queries

- **NFR4:** The tool shall order predicate evaluation by cost
  - NFR4.1: Metadata predicates evaluated first (cheapest)
  - NFR4.2: Content predicates evaluated second
  - NFR4.3: Semantic predicates evaluated last (most expensive)
  - NFR4.4: Lazy file content loading

- **NFR5:** The tool shall complete searches within reasonable time bounds
  - NFR5.1: Metadata-only queries < 1 second on typical codebases
  - NFR5.2: Content queries < 5 seconds on typical codebases
  - NFR5.3: Semantic queries < 10 seconds on typical codebases

#### Reliability

- **NFR6:** The tool shall handle non-UTF8 files gracefully without crashing
  - NFR6.1: Binary files are skipped for content predicates
  - NFR6.2: Warning message output for unreadable files
  - NFR6.3: Partial results returned even if some files fail

- **NFR7:** The tool shall provide clear, unambiguous error messages
  - NFR7.1: Parser errors include position and expected tokens
  - NFR7.2: File errors include path and reason
  - NFR7.3: All errors written to stderr

- **NFR8:** The tool shall produce deterministic output
  - NFR8.1: Identical query and filesystem state produce identical output
  - NFR8.2: Results sorted alphabetically by path
  - NFR8.3: No timing-dependent variations

#### Usability

- **NFR9:** The tool shall work out-of-the-box with zero configuration required
  - NFR9.1: Sensible defaults for all options
  - NFR9.2: Automatic language detection by extension
  - NFR9.3: Automatic terminal capability detection

- **NFR10:** The tool shall provide intuitive query syntax
  - NFR10.1: Predicate format `key:value` is self-explanatory
  - NFR10.2: Boolean operators use common symbols
  - NFR10.3: Error messages suggest corrections

#### Compatibility

- **NFR11:** The tool shall maintain backward compatibility across minor versions
  - NFR11.1: CLI interface stable within major version
  - NFR11.2: Query syntax additions are non-breaking
  - NFR11.3: Configuration format versioned

- **NFR12:** The tool shall support multiple platforms
  - NFR12.1: Linux support (x86_64, aarch64)
  - NFR12.2: macOS support (x86_64, aarch64)
  - NFR12.3: Windows support (x86_64)

#### Maintainability

- **NFR13:** The tool shall be easily extensible for new languages
  - NFR13.1: Language profiles are data-driven
  - NFR13.2: Adding language requires no core code changes
  - NFR13.3: Tree-sitter queries defined in profile

- **NFR14:** The tool shall follow Rust best practices
  - NFR14.1: clippy lints passing
  - NFR14.2: Comprehensive error handling with anyhow
  - NFR14.3: Documentation for public APIs

---

### Compatibility Requirements

- **CR1:** Existing API Compatibility
  - CR1.1: CLI interface shall remain backward compatible across minor versions
  - CR1.2: Deprecated features shall have warnings for at least one minor version
  - CR1.3: Breaking changes only in major versions

- **CR2:** Configuration Compatibility
  - CR2.1: Support both global (`~/.config/rdump/config.toml`) and local (`.rdump.toml`) config files
  - CR2.2: Local config overrides global with proper precedence
  - CR2.3: Config file format versioned for future changes

- **CR3:** Ignore File Compatibility
  - CR3.1: Respect standard `.gitignore` patterns
  - CR3.2: Provide `.rdumpignore` for tool-specific overrides
  - CR3.3: Support negation patterns for un-ignoring

- **CR4:** Platform Compatibility
  - CR4.1: Support Linux, macOS, and Windows platforms
  - CR4.2: Handle platform-specific path separators
  - CR4.3: Support Unicode filenames on all platforms

---

## Technical Constraints and Integration Requirements

### Existing Technology Stack

**Languages:** Rust (Edition 2021)

**Core Frameworks/Libraries:**

| Library | Version | Purpose |
|---------|---------|---------|
| `clap` | 4.5.4 | CLI argument parsing with derive macros |
| `pest` | 2.7.10 | Parser generator for RQL grammar |
| `rayon` | 1.10.0 | Parallel iteration and execution |
| `ignore` | 0.4.22 | Fast directory traversal with gitignore support |
| `tree-sitter` | 0.22.6 | Incremental parsing library for code awareness |
| `syntect` | 5.2.0 | Syntax highlighting |
| `serde` | 1.0.203 | Serialization framework |
| `serde_json` | 1.0.117 | JSON serialization |
| `regex` | 1.10.4 | Regular expression support |
| `chrono` | 0.4 | Date/time handling |
| `glob` | 0.3.1 | Glob pattern matching |
| `globset` | 0.4.10 | Multiple glob pattern matching |
| `anyhow` | 1.0.86 | Error handling |
| `once_cell` | 1.19.0 | Lazy static initialization |
| `dunce` | 1.0.4 | Path canonicalization |
| `toml` | 0.8.12 | TOML config parsing |
| `dirs` | 5.0.1 | Platform directories |
| `tempfile` | 3.10.1 | Temporary file handling |

**Tree-sitter Language Grammars:**

| Grammar | Version | Language |
|---------|---------|----------|
| `tree-sitter-rust` | 0.21.0 | Rust |
| `tree-sitter-python` | 0.21.0 | Python |
| `tree-sitter-javascript` | 0.21.0 | JavaScript |
| `tree-sitter-typescript` | 0.21.0 | TypeScript |
| `tree-sitter-go` | 0.21.0 | Go |
| `tree-sitter-java` | 0.21.0 | Java |

**Development Dependencies:**

| Library | Version | Purpose |
|---------|---------|---------|
| `assert_cmd` | 2.0.14 | CLI integration testing |
| `predicates` | 3.1.0 | Test assertions |

**Infrastructure:**
- Cargo/crates.io for package management
- GitHub Actions for CI/CD
- Cross-platform binary distribution

### Architecture Overview

The architecture follows a pipeline pattern with composable filters:

```
┌─────────────────┐     ┌──────────────────┐     ┌───────────────┐
│  Query String   │ ──► │  CLI Parser      │ ──► │  RQL Parser   │
│                 │     │  (clap)          │     │  (pest)       │
└─────────────────┘     └──────────────────┘     └───────┬───────┘
                                                        │
                                                        ▼
                                                ┌───────────────┐
                                                │     AST       │
                                                └───────┬───────┘
                                                        │
                        ┌───────────────────────────────┼───────────────────────────────┐
                        │                               ▼                               │
                        │                       ┌───────────────┐                       │
                        │                       │   Evaluator   │                       │
                        │                       │    Engine     │                       │
                        │                       └───────┬───────┘                       │
                        │                               │                               │
                        │           ┌───────────────────┼───────────────────┐           │
                        │           ▼                   ▼                   ▼           │
                        │   ┌───────────────┐   ┌───────────────┐   ┌───────────────┐   │
                        │   │   Metadata    │   │   Content     │   │   Semantic    │   │
                        │   │  Predicates   │   │  Predicates   │   │   Engine      │   │
                        │   │ (ignore,glob) │   │   (regex)     │   │ (tree-sitter) │   │
                        │   └───────────────┘   └───────────────┘   └───────────────┘   │
                        │                                                               │
                        │                    Parallel File Walker                       │
                        │                         (rayon)                               │
                        └───────────────────────────────┬───────────────────────────────┘
                                                        │
                                                        ▼
                                                ┌───────────────┐
                                                │ Matched Files │
                                                └───────┬───────┘
                                                        │
                                                        ▼
                                                ┌───────────────┐
                                                │   Formatter   │
                                                │   (syntect)   │
                                                └───────┬───────┘
                                                        │
                                                        ▼
                                                ┌───────────────┐
                                                │    Output     │
                                                └───────────────┘
```

### Component Details

#### 1. CLI Parser (clap)

- Declarative macro-based API for CLI structure
- Subcommands: `search` (default), `lang`, `preset`
- Automatic help generation and validation
- Type-safe argument parsing

**Detailed Responsibilities:**
- Parse command-line arguments into strongly-typed structures
- Validate argument combinations (e.g., `-C` only with `--format=hunks`)
- Generate shell completions for bash, zsh, fish
- Provide contextual help and usage examples

**Data Flow:**
```rust
fn main() -> Result<()> {
    let cli = Cli::parse();  // clap derives this

    match cli.command {
        Command::Search { query, format, .. } => {
            let ast = parse_query(&query)?;
            let results = evaluate(ast, &cli.options)?;
            format_output(results, format)?;
        }
        Command::Lang { subcommand } => { /* ... */ }
        Command::Preset { subcommand } => { /* ... */ }
    }
}
```

#### 2. RQL Parser (pest)

- Grammar defined in `src/rql.pest`
- Transforms query string to Abstract Syntax Tree
- Excellent error reporting for invalid queries
- Decouples syntax from processing logic

**Grammar Structure:**
```pest
query = { SOI ~ expr ~ EOI }
expr = { term ~ (or_op ~ term)* }
term = { factor ~ (and_op ~ factor)* }
factor = { not_op? ~ (group | predicate) }
group = { "(" ~ expr ~ ")" }
predicate = { key ~ ":" ~ value }
key = @{ ASCII_ALPHA ~ ASCII_ALPHANUMERIC* }
value = { quoted_string | regex | identifier }
```

**AST Node Types:**
```rust
enum AstNode {
    And(Box<AstNode>, Box<AstNode>),
    Or(Box<AstNode>, Box<AstNode>),
    Not(Box<AstNode>),
    Predicate { key: String, value: String },
}
```

**Error Reporting:**
```
Error: Invalid query syntax
  ext:rs & contains:
                   ^
Expected: quoted string, regex, or identifier
```

#### 3. Evaluator Engine

- Recursive AST walker
- Short-circuit evaluation for performance
- Dispatches to predicate system
- Manages evaluation context

**Evaluation Algorithm:**
```rust
fn evaluate(node: &AstNode, ctx: &mut FileContext) -> Result<bool> {
    match node {
        AstNode::And(left, right) => {
            // Short-circuit: if left is false, don't evaluate right
            if !evaluate(left, ctx)? {
                return Ok(false);
            }
            evaluate(right, ctx)
        }
        AstNode::Or(left, right) => {
            // Short-circuit: if left is true, don't evaluate right
            if evaluate(left, ctx)? {
                return Ok(true);
            }
            evaluate(right, ctx)
        }
        AstNode::Not(inner) => {
            Ok(!evaluate(inner, ctx)?)
        }
        AstNode::Predicate { key, value } => {
            let evaluator = REGISTRY.get(key)?;
            evaluator.evaluate(value, ctx)
        }
    }
}
```

**FileContext Structure:**
```rust
struct FileContext {
    path: PathBuf,
    metadata: Metadata,
    content: OnceCell<String>,      // Lazy-loaded
    tree: OnceCell<Tree>,           // Lazy-parsed
}
```

#### 4. Predicate System

- Trait-based modular design
- `PredicateEvaluator` trait for each predicate
- Runtime dispatch via HashMap registry
- Easy to add new predicates

**Trait Definition:**
```rust
trait PredicateEvaluator: Send + Sync {
    /// Evaluate predicate against file context
    fn evaluate(&self, value: &str, ctx: &mut FileContext) -> Result<bool>;

    /// Cost estimate for query optimization (lower = cheaper)
    fn cost(&self) -> u32;

    /// Human-readable description for help
    fn description(&self) -> &str;
}
```

**Predicate Registry:**
```rust
lazy_static! {
    static ref REGISTRY: HashMap<&'static str, Box<dyn PredicateEvaluator>> = {
        let mut m = HashMap::new();
        // Metadata predicates (cost: 1-10)
        m.insert("ext", Box::new(ExtPredicate));
        m.insert("name", Box::new(NamePredicate));
        m.insert("path", Box::new(PathPredicate));
        m.insert("size", Box::new(SizePredicate));
        m.insert("modified", Box::new(ModifiedPredicate));
        // Content predicates (cost: 100)
        m.insert("contains", Box::new(ContainsPredicate));
        m.insert("c", Box::new(ContainsPredicate));  // alias
        m.insert("matches", Box::new(MatchesPredicate));
        m.insert("m", Box::new(MatchesPredicate));  // alias
        // Semantic predicates (cost: 1000)
        m.insert("func", Box::new(FuncPredicate));
        m.insert("struct", Box::new(StructPredicate));
        // ... etc
        m
    };
}
```

**Adding a New Predicate:**
```rust
struct MyPredicate;

impl PredicateEvaluator for MyPredicate {
    fn evaluate(&self, value: &str, ctx: &mut FileContext) -> Result<bool> {
        // Implementation here
    }

    fn cost(&self) -> u32 { 50 }

    fn description(&self) -> &str {
        "Match files by my criteria"
    }
}

// Register in REGISTRY
m.insert("mypred", Box::new(MyPredicate));
```

#### 5. Parallel File Walker

- `ignore` crate for fast traversal
- Respects gitignore and rdumpignore
- `rayon` for parallel file processing
- Automatic work distribution

**Walking Strategy:**
```rust
fn walk_and_evaluate(root: &Path, ast: &AstNode, opts: &Options) -> Result<Vec<FileResult>> {
    let walker = WalkBuilder::new(root)
        .hidden(!opts.hidden)
        .ignore(!opts.no_ignore)
        .git_ignore(!opts.no_ignore)
        .threads(opts.threads)
        .build_parallel();

    let results = Mutex::new(Vec::new());

    walker.run(|| {
        Box::new(|entry| {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => return WalkState::Continue,
            };

            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let mut ctx = FileContext::new(entry.path());
                if evaluate(ast, &mut ctx).unwrap_or(false) {
                    results.lock().push(FileResult::from(ctx));
                }
            }

            WalkState::Continue
        })
    });

    let mut results = results.into_inner();
    results.sort_by(|a, b| a.path.cmp(&b.path));  // Deterministic order
    Ok(results)
}
```

**Ignore File Precedence:**
1. Command-line `--no-ignore` (highest)
2. `.rdumpignore` in current/parent directories
3. `.gitignore` in current/parent directories
4. Global gitignore (`~/.config/git/ignore`)
5. Built-in patterns (lowest)

#### 6. Semantic Engine

- Tree-sitter for parsing
- Language profiles map predicates to queries
- Language-agnostic core logic
- Data-driven language support

**Language Profile Structure:**
```rust
struct LanguageProfile {
    name: &'static str,
    extensions: &'static [&'static str],
    language: Language,
    queries: HashMap<&'static str, &'static str>,
}

// Example: Rust profile
lazy_static! {
    static ref RUST_PROFILE: LanguageProfile = LanguageProfile {
        name: "rust",
        extensions: &["rs"],
        language: tree_sitter_rust::language(),
        queries: hashmap! {
            "func" => "(function_item name: (identifier) @match)",
            "struct" => "(struct_item name: (type_identifier) @match)",
            "enum" => "(enum_item name: (type_identifier) @match)",
            "trait" => "(trait_item name: (type_identifier) @match)",
            "impl" => "(impl_item type: (type_identifier) @match)",
            "import" => "(use_declaration argument: (_) @match)",
            "call" => "(call_expression function: (identifier) @match)",
        },
    };
}
```

**Semantic Evaluation Flow:**
```rust
fn evaluate_semantic(predicate: &str, value: &str, ctx: &mut FileContext) -> Result<bool> {
    // 1. Determine language from extension
    let profile = get_profile_for_extension(ctx.extension())?;

    // 2. Get query string for this predicate
    let query_str = profile.queries.get(predicate)
        .ok_or_else(|| Error::PredicateNotAvailable(predicate, profile.name))?;

    // 3. Parse file (cached)
    let tree = ctx.get_or_parse_tree(&profile.language)?;

    // 4. Execute query
    let query = Query::new(profile.language, query_str)?;
    let mut cursor = QueryCursor::new();

    for match_ in cursor.matches(&query, tree.root_node(), ctx.content().as_bytes()) {
        for capture in match_.captures {
            let node_text = &ctx.content()[capture.node.byte_range()];
            if value == "." || node_text == value || glob_match(value, node_text) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
```

#### 7. Formatter

- `syntect` for syntax highlighting
- Multiple output formats
- Line number injection
- Context line support

**Formatter Trait:**
```rust
trait OutputFormatter: Send + Sync {
    fn format(&self, results: &[FileResult], opts: &FormatOptions) -> Result<String>;
}

struct FormatOptions {
    line_numbers: bool,
    context_before: usize,
    context_after: usize,
    color: ColorChoice,
    syntax_theme: String,
}
```

**Format Implementations:**

| Format | Output Structure | Use Case |
|--------|------------------|----------|
| `markdown` | Headers, metadata, fenced code blocks | Human reading, AI context |
| `json` | Array of objects with path, size, content | Programmatic processing |
| `cat` | Raw concatenated content | Piping to other tools |
| `paths` | One path per line | xargs, while read loops |
| `hunks` | Matching lines with context | Grep-like output |
| `find` | ls -l style | File listing |

**Syntax Highlighting:**
```rust
fn highlight_code(code: &str, extension: &str, theme: &str) -> String {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();

    let syntax = syntax_set.find_syntax_by_extension(extension)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let theme = &theme_set.themes[theme];

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut output = String::new();

    for line in LinesWithEndings::from(code) {
        let ranges = highlighter.highlight_line(line, &syntax_set)?;
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        output.push_str(&escaped);
    }

    output
}

### Query Optimization Guidelines

Efficient query construction is critical for rdump performance. Due to short-circuit evaluation, predicate order significantly impacts execution time.

#### Predicate Cost Tiers

| Tier | Cost | Predicates | Operation | When Evaluated |
|------|------|------------|-----------|----------------|
| 1 | 1-5 | `ext:`, `name:` | String comparison | Immediate (from path) |
| 2 | 5-10 | `path:`, `path_exact:`, `in:` | Path matching | Immediate (from path) |
| 3 | 10-20 | `size:`, `modified:` | Stat syscall | On first access |
| 4 | 100-500 | `contains:`, `matches:` | File read + search | On first content access |
| 5 | 500-2000 | `func:`, `struct:`, `def:`, `import:`, `call:` | Parse + tree query | On first semantic access |
| 6 | 1000-3000 | `comment:`, `str:` | Parse + full tree scan | On first semantic access |

#### Optimization Rules

**Rule 1: Cheapest predicates first**

Place metadata predicates before content predicates before semantic predicates.

```bash
# Bad - reads all files, then filters by extension
rdump "contains:async & ext:rs"

# Good - filters by extension first, only reads .rs files
rdump "ext:rs & contains:async"

# Best - filters by path, then extension, then content
rdump "path:src/ & ext:rs & contains:async"
```

**Rule 2: Most selective predicates first (within same tier)**

If two predicates have similar cost, put the more selective one first.

```bash
# If you have 1000 .rs files but only 10 in src/api/
rdump "path:src/api & ext:rs & func:handle"  # Better
rdump "ext:rs & path:src/api & func:handle"  # Also fine, similar cost
```

**Rule 3: Use short-circuit to skip expensive operations**

Structure queries so false results short-circuit before expensive predicates.

```bash
# Looking for large Rust files with async functions
rdump "ext:rs & size:>10kb & func:async"

# The size: check (stat) is cheaper than func: (parse)
# Files < 10kb won't be parsed at all
```

**Rule 4: Combine metadata predicates liberally**

Metadata predicates are so cheap that adding more barely impacts performance.

```bash
# This is fine - all metadata checks are nearly free
rdump "ext:rs & path:src/ & !path:test & size:<100kb & modified:<7d & func:main"
```

**Rule 5: Avoid negated semantic predicates when possible**

Negated semantic predicates (`!func:`) require parsing to prove absence.

```bash
# Expensive - must parse every .rs file to check for absence
rdump "ext:rs & !func:test"

# Better - use path exclusion if possible
rdump "ext:rs & !path:test"
```

#### Query Performance Examples

**Example 1: Find async functions in recent Rust files**

```bash
# Optimal order
rdump "ext:rs & path:src/ & modified:<1d & func:async"

# Execution:
# 1. ext:rs - filter 10,000 files → 500 .rs files
# 2. modified:<1d - stat 500 files → 50 recent files
# 3. path:src/ - filter 50 files → 40 in src/
# 4. func:async - parse 40 files → 8 matches
```

**Example 2: Find TODO comments in production code**

```bash
# Optimal order
rdump "ext:rs & !path:test & !path:examples & comment:TODO"

# Execution:
# 1. ext:rs - filter to Rust files
# 2. !path:test - exclude test files (cheap)
# 3. !path:examples - exclude examples (cheap)
# 4. comment:TODO - parse remaining files (expensive, but minimal set)
```

**Example 3: Security audit for hardcoded secrets**

```bash
# Optimal order
rdump "(ext:rs | ext:py | ext:js) & !name:*_test* & !path:test & !path:mock & str:password"

# Group metadata exclusions first, then extension filter, then semantic
```

#### Performance Anti-Patterns

| Anti-Pattern | Problem | Solution |
|--------------|---------|----------|
| `func:main & ext:rs` | Parses all files | `ext:rs & func:main` |
| `contains:x & contains:y & ext:rs` | Reads all files twice | `ext:rs & contains:x & contains:y` |
| `!func:test` on large codebase | Parses everything | Use `!path:test` or `!name:*test*` |
| No path restriction | Searches entire tree | Add `path:src/` or similar |
| `matches:/complex regex/` without filters | Expensive regex on all files | Filter with `ext:` and `path:` first |

#### Measuring Query Performance

Use `--verbose` to see timing breakdown:

```bash
rdump --verbose "ext:rs & func:main" 2>&1 | grep -E "time|files"

# Output:
# Files scanned: 10,234
# Files matched: 847
# Metadata time: 0.12s
# Content time: 0.00s
# Semantic time: 2.34s
# Total time: 2.46s
```

Use `time` for overall comparison:

```bash
# Compare predicate orders
time rdump "func:main & ext:rs" --format=count
time rdump "ext:rs & func:main" --format=count
```

#### Automatic Query Optimization (Future)

Currently, rdump evaluates predicates in the order written. Future versions may include:

- **Cost-based reordering:** Automatically reorder predicates by cost tier
- **Selectivity estimation:** Estimate which predicates filter most aggressively
- **Query plan explanation:** `--explain` flag to show execution plan

Until then, follow the manual optimization guidelines above.

---

### Code Organization and Standards

**Directory Structure:**
```
rdump/
├── src/
│   ├── main.rs                 # Entry point
│   ├── cli.rs                  # CLI definitions
│   ├── rql.pest                # RQL grammar
│   ├── parser/                 # Query parsing
│   ├── evaluator/              # AST evaluation
│   ├── predicates/             # Predicate implementations
│   │   ├── metadata/           # ext, name, path, size, modified
│   │   ├── content/            # contains, matches
│   │   └── code_aware/         # Semantic predicates
│   │       └── profiles/       # Language-specific profiles
│   ├── formatter/              # Output formatting
│   └── config/                 # Configuration handling
├── tests/                      # Integration tests
├── Cargo.toml                  # Dependencies
└── README.md                   # Documentation
```

**Naming Conventions:**
- Functions/variables: `snake_case`
- Types/traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`
- Predicates: lowercase key names

**Coding Standards:**
- All public APIs documented
- Errors handled with `anyhow`
- No `unwrap()` in library code
- Comprehensive test coverage
- clippy lints passing

**Documentation Standards:**
- README with comprehensive usage guide
- Inline documentation for public APIs
- Example queries throughout
- Architecture documentation

### Deployment and Operations

**Build Process:**
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with clippy
cargo clippy
```

**Distribution Channels:**

1. **Crates.io:**
   ```bash
   cargo install rdump
   ```

2. **GitHub Releases:**
   - Pre-compiled binaries for Linux, macOS, Windows
   - Checksums for verification

3. **Source:**
   ```bash
   git clone https://github.com/almaclaine/rdump
   cd rdump/rdump
   cargo build --release
   ```

**Configuration Management:**

- Global config: `~/.config/rdump/config.toml`
- Local config: `.rdump.toml`
- Environment variables for overrides
- Presets in config files

**Monitoring and Logging:**
- `--verbose` flag for debug output
- Errors to stderr
- Results to stdout
- Exit codes for scripting

### Risk Assessment and Mitigation

**Technical Risks:**

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Tree-sitter grammar updates break queries | Medium | High | Pin grammar versions, test against updates |
| Large file handling causes memory issues | Low | High | Lazy loading, streaming where possible |
| Complex regex patterns are slow | Medium | Medium | Short-circuit evaluation, user education |
| Parse errors in malformed code | Medium | Low | Tree-sitter graceful degradation |

**Integration Risks:**

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Shell interpretation of query operators | High | Medium | Clear documentation, error messages |
| Cross-platform path handling | Medium | Medium | dunce crate, platform testing |
| Unicode filename handling | Low | Medium | Rust native Unicode support |

**Performance Risks:**

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Expensive semantic queries on large codebases | Medium | High | Metadata-first filtering, parallelization |
| Memory usage with many large files | Low | Medium | Streaming output, bounded buffers |
| Slow startup due to grammar loading | Low | Low | Lazy grammar initialization |

---

## Epic and Story Structure

### Epic Approach

**Epic Structure Decision:** Single comprehensive epic documenting the complete rdump implementation. The project represents a cohesive tool with tightly integrated components that were developed as a unified system.

**Rationale:** While rdump has many features, they all serve the single goal of providing expressive code search. The components are tightly coupled (query language, predicates, output formats) and cannot be delivered independently in a useful way.

---

## Epic 1: Code-Aware File Search Tool

**Epic Goal:** Deliver a fast, expressive, language-aware file search CLI tool that bridges the context gap between developers/AI agents and codebases through a powerful query language and structured output.

**Integration Requirements:**
- Seamless integration with existing developer workflows (shell, editors, CI/CD)
- Compatible with standard ignore file patterns
- Structured output formats for both human and machine consumption

**Success Criteria:**
- Query language is intuitive for developers and generatable by AI
- Performance is competitive with ripgrep for text search
- Semantic search provides significant value over text-only tools
- Output formats are directly usable by target consumers

---

### Story 1.1: CLI Foundation and Query Parsing

As a **developer**,
I want **a command-line interface that accepts queries in a custom language**,
so that **I can express complex file search criteria in a single command**.

#### Acceptance Criteria

1. CLI accepts a query string as the primary positional argument
2. RQL parser correctly handles boolean operators (`&`, `|`, `!`) with proper precedence
3. Parser supports grouping with parentheses to any nesting depth
4. Parser handles quoted values for strings with special characters
5. Clear error messages are displayed for invalid query syntax with position information
6. Help text documents all available flags and options
7. Version information is accessible via `--version`
8. Subcommands (`search`, `lang`, `preset`) are properly routed

#### Technical Notes

- Use `clap` derive macros for CLI definition
- Use `pest` PEG parser for RQL grammar
- Grammar defined in separate `.pest` file for maintainability
- AST nodes should be serializable for verbose output

#### Integration Verification

- IV1: Existing shell workflows unaffected - queries must be properly quoted
- IV2: Error messages are actionable and point to exact syntax issues
- IV3: Performance baseline established for query parsing (<10ms)

---

### Story 1.2: Metadata Predicate Implementation

As a **developer**,
I want **to filter files by metadata like extension, size, and modification time**,
so that **I can quickly narrow down search results without reading file contents**.

#### Acceptance Criteria

1. `ext:` predicate matches file extensions case-insensitively
2. `name:` predicate matches filename with glob patterns
3. `path:` predicate matches substring anywhere in full path
4. `path_exact:` predicate requires exact path match
5. `size:` predicate supports operators (`>`, `<`, `=`, `>=`, `<=`) and units (b, kb, mb, gb)
6. `modified:` predicate supports time-based filtering with units (s, m, h, d, w)
7. `in:` predicate matches files in specific directory
8. All metadata predicates are evaluated before content predicates for performance

#### Technical Notes

- Use `glob` crate for pattern matching
- Use `chrono` for time calculations
- Metadata predicates should not read file contents
- Results should be cached in `FileContext` struct

#### Edge Cases

- Symlinks: Follow by default, provide option to skip
- Empty extensions: Handle files with no extension
- Future modification times: Handle clock skew gracefully

#### Integration Verification

- IV1: Metadata predicates work correctly on all supported platforms (Linux, macOS, Windows)
- IV2: Glob patterns follow standard gitignore conventions
- IV3: Size/time comparisons handle edge cases (zero size, very old files) correctly

---

### Story 1.3: Content Search Predicates

As a **developer**,
I want **to search within file contents using literal strings and regex**,
so that **I can find files containing specific code patterns**.

#### Acceptance Criteria

1. `contains:`/`c:` performs case-sensitive literal substring search
2. `matches:`/`m:` performs regex search using Rust regex crate
3. Content is only read if metadata predicates pass (lazy loading)
4. Non-UTF8 files are handled gracefully without crashes
5. Binary files are detected and skipped with appropriate message
6. Short-circuit evaluation skips content checks when possible
7. Large files are handled without excessive memory usage

#### Technical Notes

- Use Rust `regex` crate for pattern matching
- Implement lazy content loading in `FileContext`
- Use memory-mapped files for large file handling
- Detect binary files by checking for null bytes

#### Edge Cases

- Empty files: Match only `!contains:` patterns
- Very large files (>100MB): Consider streaming or sampling
- Files with mixed encodings: Best-effort UTF8 conversion

#### Integration Verification

- IV1: Content search performance is acceptable (<100ms per file on average)
- IV2: Regex syntax follows Rust regex crate conventions (documented)
- IV3: Memory usage remains bounded during content scanning (<1GB)

---

### Story 1.4: Parallel File Discovery and Processing

As a **developer**,
I want **the tool to utilize all CPU cores for file processing**,
so that **searches complete quickly even on large codebases**.

#### Acceptance Criteria

1. Directory traversal uses `ignore` crate for parallel walking
2. File evaluation uses `rayon` for parallel processing
3. `.gitignore` patterns are respected by default
4. `.rdumpignore` patterns take precedence over `.gitignore`
5. `--no-ignore` flag disables all ignore file logic
6. `--hidden` flag includes hidden files/directories
7. `--threads` option controls worker thread count
8. Results are sorted alphabetically for determinism
9. Nested ignore files in subdirectories are respected

#### Technical Notes

- `ignore` crate handles gitignore parsing and parallel traversal
- `rayon` provides work-stealing thread pool
- Results collected in concurrent data structure, then sorted
- Default thread count is number of logical CPU cores

#### Edge Cases

- Circular symlinks: Detect and skip
- Permission denied: Log warning, continue
- Very deep directory trees: No stack overflow

#### Integration Verification

- IV1: Parallel processing correctly handles filesystem edge cases
- IV2: Ignore patterns from nested directories work correctly
- IV3: Thread safety maintained throughout pipeline (no data races)

---

### Story 1.5: Code-Aware Semantic Predicates (Core)

As a **developer**,
I want **to search for code structures like functions, classes, and imports**,
so that **I can find definitions and usages without false positives from text matching**.

#### User Story Details

**Estimated Effort:** 5 story points
**Dependencies:** Story 1.1 (CLI Foundation), Story 1.4 (Parallel Processing)
**Risk Level:** High (complex tree-sitter integration)

#### Acceptance Criteria

| ID | Criterion | Verification Method |
|----|-----------|-------------------|
| AC1 | Tree-sitter integration parses code into syntax trees | Unit test with sample code |
| AC2 | `def:` finds generic definitions (class, struct, trait, enum, etc.) | Integration test per language |
| AC3 | `func:` finds function/method definitions | Integration test per language |
| AC4 | `import:` finds import/use/require statements | Integration test per language |
| AC5 | `call:` finds function/method call sites | Integration test per language |
| AC6 | `comment:` finds text within any comment node | Integration test |
| AC7 | `str:` finds text within any string literal node | Integration test |
| AC8 | Wildcard `.` value matches any occurrence | Unit test |
| AC9 | Language detection based on file extension | Unit test with mapping table |
| AC10 | Graceful handling of syntax errors (partial parse) | Integration test with malformed code |
| AC11 | Predicate evaluation returns correct line numbers | Output verification |
| AC12 | Multiple semantic predicates on same file share parsed tree | Performance test |

#### Detailed Acceptance Criteria

**AC1: Tree-sitter Integration**
- Given a supported source file
- When the file is processed with a semantic predicate
- Then tree-sitter parses the file into a complete syntax tree
- And the parse completes within 500ms for files under 1MB
- And memory usage for parsing is bounded

**AC2: Definition Finding (`def:` predicate)**
- Given a query `def:User`
- When executed against code containing `struct User`, `class User`, `enum User`, or `type User`
- Then the predicate matches and returns the file
- And the match includes the correct line number
- And does NOT match `User` in comments, strings, or variable names

**AC3: Function Finding (`func:` predicate)**
- Given a query `func:process_data`
- When executed against code containing function definitions
- Then it matches:
  - Rust: `fn process_data()`, `pub fn process_data()`, `async fn process_data()`
  - Python: `def process_data():`, methods in classes
  - JavaScript: `function process_data()`, arrow functions assigned to `process_data`
- And does NOT match function calls to `process_data()`

**AC4: Import Finding (`import:` predicate)**
- Given a query `import:serde`
- When executed against code containing import statements
- Then it matches:
  - Rust: `use serde::Serialize;`, `use serde;`
  - Python: `import serde`, `from serde import X`
  - JavaScript: `import serde from 'serde'`, `const serde = require('serde')`

**AC5: Call Site Finding (`call:` predicate)**
- Given a query `call:process_data`
- When executed against code containing function calls
- Then it matches all invocations of `process_data(...)`
- And does NOT match function definitions
- And includes method calls like `obj.process_data()`

**AC8: Wildcard Matching**
- Given a query `func:.`
- When executed
- Then it matches ANY function definition in the file
- And can be negated: `!func:.` finds files with no functions

**AC10: Error Handling**
- Given malformed code (syntax errors)
- When tree-sitter parses the file
- Then it produces a partial syntax tree
- And semantic predicates return partial results (what could be parsed)
- And a warning is logged but execution continues

#### Technical Notes

**Architecture:**
```
Semantic Predicate Evaluation Flow:
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│ Predicate   │ ──► │ Language     │ ──► │ Tree-sitter │
│ (func:main) │     │ Profile      │     │ Parser      │
└─────────────┘     │ Registry     │     └──────┬──────┘
                    └──────────────┘            │
                                               ▼
                    ┌──────────────┐     ┌─────────────┐
                    │ Match        │ ◄── │ Query       │
                    │ Results      │     │ Execution   │
                    └──────────────┘     └─────────────┘
```

**Implementation Details:**
- Tree-sitter grammars are embedded in binary via build.rs
- Language profiles map predicate names to tree-sitter query strings
- Queries use tree-sitter query language with `@match` capture
- Cache parsed trees for multiple predicates on same file
- Use `parking_lot` for thread-safe caching

**Tree-sitter Query Examples:**

Rust function definition:
```scheme
(function_item
  name: (identifier) @match)
```

Python class definition:
```scheme
(class_definition
  name: (identifier) @match)
```

JavaScript import:
```scheme
[
  (import_statement
    source: (string) @match)
  (call_expression
    function: (identifier) @fn
    arguments: (arguments (string) @match)
    (#eq? @fn "require"))
]
```

#### Edge Cases

| Edge Case | Expected Behavior | Test |
|-----------|-------------------|------|
| Malformed code | Partial tree, partial matches, warning | `fn incomplete(` |
| Unknown language | Skip semantic predicates, use content only | `.xyz` file |
| Very large files (>10MB) | Timeout warning, skip file | Generated test file |
| Empty file | No matches, no error | Empty `.rs` file |
| Binary file | Skip semantic predicates | Compiled binary |
| Mixed language (embedded) | Match outer language only | SQL in Python string |
| Unicode identifiers | Match correctly | `fn привет()` |
| Nested definitions | Match all levels | Closure inside function |

#### Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Parse time per file | < 500ms | Benchmark |
| Memory per parsed tree | < 50MB | Memory profiler |
| Query execution | < 10ms per query per file | Benchmark |
| Cache hit rate | > 90% for multi-predicate queries | Instrumentation |

#### Definition of Done

- [ ] All acceptance criteria pass
- [ ] Unit tests for each predicate with >90% coverage
- [ ] Integration tests for each supported language
- [ ] Performance benchmarks established and passing
- [ ] Edge case tests documented and passing
- [ ] Error messages reviewed and documented
- [ ] Documentation updated with predicate reference
- [ ] Code reviewed and approved
- [ ] No regressions in existing tests

#### Integration Verification

- IV1: Semantic predicates only activate for supported languages
  - Test: Query with `func:` on `.txt` file returns no results (not error)
- IV2: Parse errors handled gracefully with useful messages
  - Test: Malformed code produces warning on stderr, partial results on stdout
- IV3: Performance acceptable with code parsing overhead (<500ms per file)
  - Test: Benchmark suite includes semantic predicate performance

#### Technical Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Tree-sitter grammar bugs | Medium | High | Pin grammar versions, report upstream |
| Performance regression | Medium | Medium | Benchmark suite, caching |
| Memory leak in parser | Low | High | Use valgrind in CI |
| Query language changes | Low | Medium | Version-lock tree-sitter |

#### Dependencies

**Upstream:**
- `tree-sitter` crate (0.22.6)
- Language grammar crates (rust, python, javascript, typescript, go, java)

**Downstream:**
- Story 1.6: Language-specific predicates build on this
- Story 1.7: React predicates build on JavaScript grammar

---

### Story 1.6: Language-Specific Semantic Predicates

As a **developer**,
I want **language-specific predicates like `struct:` for Rust and `class:` for Python**,
so that **I can make precise queries for each language's constructs**.

#### Acceptance Criteria

1. **Rust:** `struct:`, `enum:`, `trait:`, `impl:`, `type:`, `macro:`
2. **Python:** `class:`, `func:` (including methods)
3. **JavaScript:** `class:`, `func:`
4. **TypeScript:** `class:`, `interface:`, `enum:`, `type:`
5. **Go:** `struct:`, `interface:`, `type:`
6. **Java:** `class:`, `interface:`, `enum:`
7. `comment:` and `str:` work for all languages
8. Predicates unavailable for a language produce clear error message

#### Technical Notes

- Each language has a `LanguageProfile` struct
- Profile maps predicate names to tree-sitter query strings
- Queries capture node with `@match`, check identifier against value
- Profiles registered in central registry by extension

#### Language Profile Example (Rust):

```rust
struct RustProfile {
    queries: HashMap<&'static str, &'static str>,
}

// "struct" predicate for Rust
"(struct_item name: (type_identifier) @match)"

// "impl" predicate for Rust
"(impl_item type: (type_identifier) @match)"
```

#### Integration Verification

- IV1: Language profiles correctly map predicates to tree-sitter queries
- IV2: Predicate availability matches language capabilities
- IV3: `rdump lang describe` accurately shows available predicates per language

---

### Story 1.7: React-Specific Predicates

As a **React developer**,
I want **predicates for React concepts like components, hooks, and JSX elements**,
so that **I can analyze my React codebase structure**.

#### Acceptance Criteria

1. `component:` finds React component definitions (function, class, arrow, memo)
2. `element:` finds JSX element tags by name
3. `hook:` finds hook calls (any function starting with `use`)
4. `customhook:` finds custom hook definitions (function definitions starting with `use`)
5. `prop:` finds props passed to JSX elements
6. Predicates work on `.jsx` and `.tsx` files
7. Handles both default and named exports
8. Handles both PascalCase components and lowercase elements

#### Technical Notes

- React predicates use JavaScript/TypeScript tree-sitter grammars
- JSX is parsed as part of JS/TS grammar
- Component detection requires multiple patterns (function, class, arrow, memo wrapper)
- Hook pattern: `/^use[A-Z]/`

#### Tree-sitter Query Examples:

```scheme
;; Component (function declaration)
(function_declaration
  name: (identifier) @match
  (#match? @match "^[A-Z]"))

;; Hook call
(call_expression
  function: (identifier) @match
  (#match? @match "^use[A-Z]"))

;; JSX element
(jsx_element
  open_tag: (jsx_opening_element
    name: (identifier) @match))
```

#### Integration Verification

- IV1: React predicates correctly identify all component patterns
- IV2: Hook detection handles standard hooks (useState, useEffect) and custom hooks
- IV3: JSX parsing handles complex nested structures and fragments

---

### Story 1.8: Multiple Output Formats

As a **developer**,
I want **multiple output formats optimized for different use cases**,
so that **I can consume results in my preferred way or pipe to other tools**.

#### Acceptance Criteria

1. **`markdown`:** File path header, metadata (size, modified), fenced code block with language
2. **`json`:** Array of objects with `path`, `size_bytes`, `modified_iso8601`, `content`
3. **`cat`:** Raw concatenated file contents, optional line numbers
4. **`paths`:** One file path per line, suitable for `xargs`
5. **`hunks`:** Only matching lines/blocks with optional context
6. **`find`:** ls -l style with permissions, size, date, path
7. `--line-numbers` prepends line numbers in applicable formats
8. JSON `content` field always contains original unmodified content
9. Syntax highlighting in terminal for applicable formats

#### Format Examples:

**Markdown:**
```markdown
---
File: src/main.rs
Size: 1.2 KB
Modified: 2024-01-15T10:30:00Z
---
```rust
fn main() {
    println!("Hello, rdump!");
}
```
```

**JSON:**
```json
[
  {
    "path": "/home/user/project/src/main.rs",
    "size_bytes": 1234,
    "modified_iso8601": "2024-01-15T10:30:00Z",
    "content": "fn main() {\n    println!(\"Hello, rdump!\");\n}\n"
  }
]
```

**Paths:**
```
/home/user/project/src/main.rs
/home/user/project/src/lib.rs
```

#### Technical Notes

- Use `syntect` for syntax highlighting
- JSON output must be valid JSON (escape special characters)
- Line numbers right-aligned with padding
- Context lines marked differently from match lines

#### Integration Verification

- IV1: All formats produce valid, parseable output
- IV2: Format selection doesn't affect search performance
- IV3: Output is suitable for piping to other tools (xargs, jq, etc.)

---

### Story 1.9: Configuration and Presets

As a **developer**,
I want **to save common queries as presets and configure defaults**,
so that **I can quickly run frequent searches without retyping**.

#### Acceptance Criteria

1. Global config at `~/.config/rdump/config.toml`
2. Local config at `.rdump.toml` in project root (or any parent)
3. Local config values override global config
4. `rdump preset list` shows all presets with their queries
5. `rdump preset add <name> <query>` creates or updates preset
6. `rdump preset remove <name>` deletes preset
7. `--preset` or `-p` flag uses saved preset in query
8. Multiple presets can be combined: `-p rust-src -p no-tests`
9. Presets expand inline and can be combined with additional predicates

#### Configuration File Format:

```toml
# ~/.config/rdump/config.toml

[defaults]
format = "hunks"
color = "auto"
context = 3

[presets]
rust-src = "ext:rs & !path:tests/ & !path:target/"
js-check = "(ext:js | ext:jsx | ext:ts | ext:tsx) & !path:node_modules/"
recent = "modified:<7d"
large = "size:>50kb"
```

#### Usage Examples:

```bash
# Use preset
rdump -p rust-src "func:main"

# Combine presets
rdump -p rust-src -p recent "struct:Config"

# Preset with additional predicates
rdump -p js-check "& hook:useState"
```

#### Technical Notes

- Use `toml` crate for parsing
- Use `dirs` crate for platform-specific config location
- Presets are simple string substitution
- Invalid preset names produce clear error

#### Integration Verification

- IV1: Config file parsing handles malformed files gracefully with error messages
- IV2: Preset precedence (local over global) works correctly
- IV3: Preset names don't conflict with predicate keys

---

### Story 1.10: Syntax Highlighting and User Experience

As a **developer**,
I want **syntax-highlighted output and helpful introspection commands**,
so that **results are easy to read and I can discover tool capabilities**.

#### Acceptance Criteria

1. Syntax highlighting via syntect in `markdown`, `hunks`, `cat` formats
2. `--color` flag controls highlighting: `always`, `never`, `auto`
3. Auto-detection based on terminal capabilities (isatty, TERM)
4. `rdump lang list` shows table of supported languages and extensions
5. `rdump lang describe <language>` shows all available predicates for that language
6. Context lines (`-C`, `-B`, `-A`) available in hunks format
7. Verbose mode (`-v`) shows parsed AST, timing, and debug information
8. Progress indication for long-running queries

#### Lang Commands Output:

**`rdump lang list`:**
```
Language      Extensions
-----------   ----------
Rust          .rs
Python        .py
JavaScript    .js, .jsx
TypeScript    .ts, .tsx
Go            .go
Java          .java
```

**`rdump lang describe rust`:**
```
Language: Rust
Extensions: .rs

Metadata Predicates:
  ext, name, path, path_exact, size, modified, in

Content Predicates:
  contains (c), matches (m)

Semantic Predicates:
  def, func, import, call, comment, str
  struct, enum, trait, impl, type, macro
```

#### Technical Notes

- Use `syntect` with Sublime Text syntax definitions
- Theme selection via config or environment variable
- Verbose output to stderr, results to stdout
- Timing information in verbose mode

#### Integration Verification

- IV1: Highlighting works correctly across terminals (256 color, true color)
- IV2: Language introspection matches actual capabilities
- IV3: Verbose output helps debugging query issues

---

### Story 1.11: Error Handling and Edge Cases

As a **developer**,
I want **robust error handling and graceful degradation**,
so that **the tool is reliable even with unexpected input**.

#### Acceptance Criteria

1. Invalid query syntax produces error with position and suggestion
2. File permission errors are logged and skipped, not fatal
3. Binary files are detected and skipped for content predicates
4. Non-UTF8 text files are handled with lossy conversion
5. Circular symlinks are detected and skipped
6. Very large files (>100MB) are handled without crashing
7. Keyboard interrupt (Ctrl+C) stops cleanly
8. Exit codes indicate success (0), no matches (1), or error (2)

#### Error Message Examples:

**Query syntax error:**
```
Error: Invalid query syntax at position 15
  ext:rs & contains:
                   ^
Expected: quoted string or identifier
Hint: Did you mean 'contains:"some text"'?
```

**File permission error:**
```
Warning: Permission denied: /etc/shadow (skipping)
```

#### Technical Notes

- Use `anyhow` for error handling with context
- All errors to stderr, results to stdout
- Graceful degradation: partial results better than none
- Signal handling for clean shutdown

#### Integration Verification

- IV1: Tool never panics on any input
- IV2: Partial results returned when some files fail
- IV3: Error messages are actionable and helpful

---

### Story 1.12: Performance Optimization

As a **developer**,
I want **fast search performance even on large codebases**,
so that **I can use rdump interactively without waiting**.

#### Acceptance Criteria

1. Metadata-only queries complete in <1 second on 100K files
2. Content queries complete in <5 seconds on 10K text files
3. Semantic queries complete in <10 seconds on 10K source files
4. Memory usage stays under 1GB for typical queries
5. CPU utilization scales with available cores
6. No unnecessary file reads (lazy loading)
7. Results stream to output as found (where format allows)

#### Optimization Strategies

1. **Predicate Ordering:** Cheap predicates first
2. **Short-Circuit Evaluation:** Skip remaining predicates when possible
3. **Lazy Loading:** Only read content when needed
4. **Parallel Processing:** Distribute work across cores
5. **Ignore Optimization:** Skip ignored directories early
6. **Caching:** Reuse parsed trees for multiple semantic predicates

#### Technical Notes

- Profile with `cargo flamegraph`
- Benchmark with `criterion`
- Test with real-world codebases (Linux kernel, Chromium)
- Monitor memory with `heaptrack`

#### Integration Verification

- IV1: Performance targets met on reference codebases
- IV2: No performance regression between versions
- IV3: Memory usage bounded and predictable

---

## Testing Strategy

### Testing Philosophy

rdump follows a comprehensive testing approach that ensures reliability across all supported platforms and use cases. Testing is not an afterthought but an integral part of the development process.

### Test Categories

#### Unit Tests

**Scope:** Individual functions and modules in isolation

**Coverage Areas:**
- RQL parser grammar and AST generation
- Individual predicate evaluation logic
- Output formatter implementations
- Configuration file parsing
- Utility functions

**Tools:**
- Rust's built-in `#[test]` attribute
- `assert_eq!`, `assert!`, `assert_ne!` macros
- Property-based testing with `proptest` (future)

**Example Unit Tests:**

```rust
#[test]
fn test_ext_predicate_case_insensitive() {
    let pred = ExtPredicate::new("rs");
    assert!(pred.evaluate("file.rs"));
    assert!(pred.evaluate("file.RS"));
    assert!(pred.evaluate("file.Rs"));
    assert!(!pred.evaluate("file.txt"));
}

#[test]
fn test_size_predicate_parsing() {
    assert_eq!(parse_size(">10kb"), Ok((Operator::Gt, 10240)));
    assert_eq!(parse_size("<=5mb"), Ok((Operator::Lte, 5242880)));
    assert!(parse_size("invalid").is_err());
}

#[test]
fn test_rql_parser_precedence() {
    let ast = parse("ext:rs & name:test | ext:py").unwrap();
    // Should parse as (ext:rs & name:test) | ext:py
    assert!(matches!(ast, AstNode::Or(_, _)));
}
```

#### Integration Tests

**Scope:** Multiple components working together

**Coverage Areas:**
- Full query execution pipeline
- File system interaction
- Output format correctness
- Configuration loading and merging
- Ignore file handling

**Tools:**
- `assert_cmd` for CLI testing
- `predicates` for output assertions
- `tempfile` for isolated test environments

**Example Integration Tests:**

```rust
#[test]
fn test_full_query_execution() {
    let dir = create_test_directory();
    write_file(&dir, "src/main.rs", "fn main() {}");
    write_file(&dir, "src/lib.rs", "pub fn helper() {}");
    write_file(&dir, "tests/test.rs", "fn test_it() {}");

    let output = Command::cargo_bin("rdump")
        .unwrap()
        .args(&["ext:rs & !path:tests", "--format=paths"])
        .current_dir(&dir)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("src/main.rs"));
    assert!(stdout.contains("src/lib.rs"));
    assert!(!stdout.contains("tests/test.rs"));
}

#[test]
fn test_json_output_format() {
    let dir = create_test_directory();
    write_file(&dir, "test.rs", "fn main() {}");

    let output = Command::cargo_bin("rdump")
        .unwrap()
        .args(&["ext:rs", "--format=json"])
        .current_dir(&dir)
        .output()
        .unwrap();

    let json: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json.len(), 1);
    assert!(json[0]["path"].as_str().unwrap().ends_with("test.rs"));
    assert!(json[0]["content"].as_str().unwrap().contains("fn main()"));
}
```

#### Semantic Predicate Tests

**Scope:** Tree-sitter integration and language-specific predicates

**Coverage Areas:**
- Correct parsing of each supported language
- Accurate predicate matching
- Edge cases in language syntax
- Graceful handling of malformed code

**Example Semantic Tests:**

```rust
#[test]
fn test_rust_struct_predicate() {
    let code = r#"
        struct User { name: String }
        struct Config { debug: bool }
        fn user_helper() {}
    "#;

    let matches = evaluate_predicate("struct:User", code, "rust");
    assert_eq!(matches.len(), 1);
    assert!(matches[0].contains("struct User"));
}

#[test]
fn test_react_hook_predicate() {
    let code = r#"
        function Component() {
            const [state, setState] = useState(0);
            useEffect(() => {}, []);
            return <div>{state}</div>;
        }
    "#;

    let matches = evaluate_predicate("hook:useState", code, "tsx");
    assert_eq!(matches.len(), 1);
}

#[test]
fn test_malformed_code_handling() {
    let code = "fn incomplete(";  // Missing closing paren and body

    // Should not panic, should return partial results or empty
    let result = evaluate_predicate("func:incomplete", code, "rust");
    assert!(result.is_ok());
}
```

#### Performance Tests

**Scope:** Execution time and resource usage

**Coverage Areas:**
- Query execution time benchmarks
- Memory usage under load
- Scaling with file count
- Parallel processing efficiency

**Tools:**
- `criterion` for micro-benchmarks
- Custom timing harnesses
- Memory profiling with `heaptrack`

**Benchmark Examples:**

```rust
fn bench_metadata_query(c: &mut Criterion) {
    let dir = create_large_test_directory(10000); // 10K files

    c.bench_function("metadata_only_10k", |b| {
        b.iter(|| {
            Command::cargo_bin("rdump")
                .unwrap()
                .args(&["ext:rs", "--format=count"])
                .current_dir(&dir)
                .output()
                .unwrap()
        })
    });
}

fn bench_content_query(c: &mut Criterion) {
    let dir = create_large_test_directory(1000); // 1K files

    c.bench_function("content_search_1k", |b| {
        b.iter(|| {
            Command::cargo_bin("rdump")
                .unwrap()
                .args(&["contains:fn", "--format=count"])
                .current_dir(&dir)
                .output()
                .unwrap()
        })
    });
}
```

#### Platform Tests

**Scope:** Cross-platform compatibility

**Coverage Areas:**
- Path handling (separators, Unicode)
- File permissions
- Symlink behavior
- Terminal detection

**CI Matrix:**
- Linux (Ubuntu latest, x86_64)
- macOS (latest, x86_64 and aarch64)
- Windows (latest, x86_64)

### Test Data Management

#### Test Fixtures

Predefined test codebases for consistent testing:

```
tests/fixtures/
├── rust_project/         # Standard Rust project structure
├── python_project/       # Python with various patterns
├── react_project/        # React/TypeScript components
├── mixed_project/        # Multiple languages
├── large_project/        # Performance testing (generated)
├── edge_cases/           # Unusual file names, encodings
└── malformed/            # Syntax errors, binary files
```

#### Generated Test Data

For performance testing, generate realistic codebases:

```rust
fn generate_test_codebase(file_count: usize, avg_size: usize) -> TempDir {
    // Generate files with realistic distribution of:
    // - File sizes
    // - Directory depths
    // - Language mix
    // - Code patterns (functions, classes, imports)
}
```

### Continuous Integration

#### CI Pipeline Stages

1. **Lint:** `cargo clippy -- -D warnings`
2. **Format:** `cargo fmt -- --check`
3. **Unit Tests:** `cargo test --lib`
4. **Integration Tests:** `cargo test --test '*'`
5. **Documentation:** `cargo doc --no-deps`
6. **Build Release:** `cargo build --release`
7. **Cross-Platform:** Matrix build for all platforms

#### Quality Gates

- All tests must pass
- No clippy warnings
- Code coverage > 80%
- No performance regressions > 10%
- Documentation builds without warnings

### Test Coverage Goals

| Component | Target Coverage |
|-----------|----------------|
| RQL Parser | 95% |
| Predicates (Metadata) | 90% |
| Predicates (Content) | 90% |
| Predicates (Semantic) | 85% |
| Output Formatters | 90% |
| Configuration | 85% |
| CLI Handling | 80% |
| **Overall** | **85%** |

---

## Security Considerations

### Threat Model

rdump is a local CLI tool that reads files from the filesystem. The primary security considerations are:

1. **Path Traversal:** Preventing access to files outside intended scope
2. **Resource Exhaustion:** Handling large files and deep directories
3. **Sensitive Data Exposure:** Avoiding leaking secrets in output
4. **Malicious Input:** Handling crafted queries and file contents

### Security Measures

#### Path Handling

- All paths are canonicalized using `dunce` crate
- Symlinks are followed but cycles are detected
- `--root` option restricts search to specified directory
- No path escaping via `../` sequences

```rust
// Safe path handling
fn safe_canonicalize(path: &Path, root: &Path) -> Result<PathBuf> {
    let canonical = dunce::canonicalize(path)?;
    if !canonical.starts_with(root) {
        return Err(Error::PathOutsideRoot);
    }
    Ok(canonical)
}
```

#### Resource Limits

- Maximum file size for content reading (configurable, default 100MB)
- Maximum directory depth (configurable, default 100)
- Timeout for long-running operations
- Memory-mapped files for large file handling

```rust
// Resource limiting
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MB
const MAX_DEPTH: usize = 100;

fn should_read_file(metadata: &Metadata) -> bool {
    metadata.len() <= MAX_FILE_SIZE
}
```

#### Regex Safety

- Regex compilation timeout to prevent ReDoS
- Maximum regex complexity limits
- Pre-compiled common patterns

```rust
// Safe regex handling
fn compile_regex_safe(pattern: &str) -> Result<Regex> {
    RegexBuilder::new(pattern)
        .size_limit(10 * 1024 * 1024) // 10 MB state limit
        .build()
        .map_err(|e| Error::InvalidRegex(e))
}
```

#### Output Sanitization

- JSON output properly escapes special characters
- No execution of file contents
- Binary file detection and skipping

### Sensitive Data Handling

rdump may encounter sensitive data in codebases. Best practices:

#### For Users

1. **Use ignore files:** Add sensitive files to `.gitignore` or `.rdumpignore`
2. **Limit scope:** Use `--root` to restrict search area
3. **Review output:** Check results before sharing, especially JSON format
4. **Pipe carefully:** Be cautious when piping to network tools

#### For the Tool

1. **No telemetry:** rdump does not send data externally
2. **No caching:** File contents are not cached to disk
3. **Stderr for errors:** Sensitive paths only in error messages
4. **Memory clearing:** Sensitive data cleared after use (future)

### Security Audit Checklist

| Item | Status | Notes |
|------|--------|-------|
| Path traversal prevention | ✓ | Canonicalization + root check |
| Symlink cycle detection | ✓ | ignore crate handles |
| ReDoS prevention | ✓ | Regex size limits |
| Memory exhaustion | ✓ | File size limits |
| Stack overflow | ✓ | Iterative directory walking |
| Binary file handling | ✓ | Skip for content predicates |
| Error message safety | ⚠ | Review for path leakage |
| Dependency audit | ⚠ | cargo-audit in CI |

### Dependency Security

- Regular `cargo audit` checks in CI
- Dependabot alerts enabled
- Minimal dependency philosophy
- Preference for well-maintained crates

---

*Note: Future roadmap has been moved to `docs/TODOS.md`*

---

## Known Limitations

### Current Limitations

#### Language Support

| Limitation | Impact | Workaround |
|------------|--------|------------|
| Limited to 6 languages | Can't use semantic predicates for other languages | Use content predicates (`contains`, `matches`) |
| No C/C++ support | Large ecosystem unsupported | Planned for v0.3.0 |
| No embedded language support | Can't search SQL in Python strings | Use `str:` predicate for partial support |
| JSX in .js files not detected | Must use .jsx extension | Rename files or use content predicates |

#### Query Language

| Limitation | Impact | Workaround |
|------------|--------|------------|
| No variables | Can't reuse sub-expressions | Use presets for common patterns |
| No negation lookahead | Complex exclusion patterns difficult | Use multiple queries |
| No fuzzy matching | Exact names required | Use glob patterns in `name:` |
| Case-sensitive content search | May miss matches | Use regex with `(?i)` flag |

#### Performance

| Limitation | Impact | Workaround |
|------------|--------|------------|
| No incremental search | Full scan each time | Will be addressed with caching |
| Memory usage with many large files | May hit system limits | Use `size:<` to limit |
| Slow startup with many languages | First query slower | Lazy loading planned |
| No query optimization | Suboptimal predicate order | Manually order cheap predicates first |

#### Output

| Limitation | Impact | Workaround |
|------------|--------|------------|
| No streaming JSON | Must buffer all results | Use `paths` format and post-process |
| No diff output | Can't compare file versions | Pipe to diff tools |
| No grouping/aggregation | Can't group by directory | Post-process with shell tools |
| Fixed sort order | Always alphabetical | Post-process to re-sort |

### Platform-Specific Limitations

#### Windows

- Path length limited to 260 characters by default
- Some Unicode filenames may not display correctly
- Symlink support requires admin or developer mode

#### macOS

- Case-insensitive filesystem by default may cause issues
- Gatekeeper may block unsigned binaries

#### Linux

- No known platform-specific limitations

### Known Issues

| Issue | Severity | Status | Tracking |
|-------|----------|--------|----------|
| Tree-sitter memory leak with very large files | Medium | Investigating | #123 |
| Regex timeout not enforced on all platforms | Low | Open | #145 |
| `modified:` predicate timezone handling | Low | Open | #167 |
| JSON output escaping of Unicode | Low | Fixed in 0.1.8 | #189 |

### Not Planned

These features are explicitly out of scope:

1. **File modification** - rdump is read-only by design
2. **Real-time watching** - Use `watchman` or `fswatch` + rdump
3. **GUI** - CLI-first philosophy, IDE plugins planned instead
4. **Network search** - Local filesystem only
5. **Version control integration** - Use with `git ls-files` for tracked files

---

## Success Metrics

### Adoption Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Crates.io downloads | 1,000/month | Crates.io stats |
| GitHub stars | 500 | GitHub |
| Active users | 100/month | Opt-in telemetry |

### Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Metadata query (100K files) | <1s | Automated benchmark |
| Content query (10K files) | <5s | Automated benchmark |
| Semantic query (10K files) | <10s | Automated benchmark |
| Memory usage | <1GB | Automated benchmark |

### Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test coverage | >80% | cargo-tarpaulin |
| Clippy warnings | 0 | CI pipeline |
| Documentation coverage | 100% public APIs | cargo-doc |
| Issue resolution time | <7 days | GitHub |

### User Satisfaction

| Metric | Target | Measurement |
|--------|--------|-------------|
| Query accuracy | >95% | User surveys |
| Error message helpfulness | >4/5 | User surveys |
| Documentation clarity | >4/5 | User surveys |

---

## Glossary

| Term | Definition |
|------|------------|
| **AST** | Abstract Syntax Tree - structured representation of parsed query |
| **CST** | Concrete Syntax Tree - full syntax tree from tree-sitter |
| **Predicate** | A single condition in RQL (e.g., `ext:rs`) |
| **RQL** | rdump Query Language - the boolean query syntax |
| **Semantic Predicate** | Predicate that understands code structure (e.g., `func:`, `struct:`) |
| **Tree-sitter** | Incremental parsing library for code analysis |
| **Language Profile** | Configuration mapping rdump predicates to tree-sitter queries |
| **Short-circuit Evaluation** | Skipping evaluation of remaining operands when result is determined |
| **Glob Pattern** | File pattern with wildcards (e.g., `*.rs`, `src/**/*.ts`) |
| **Hunk** | A contiguous block of matching lines with context |

---
