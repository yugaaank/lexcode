
# Codelex Implementation Plan

# Phase 0. Project Foundation

## Goal

Establish architecture, repo structure, tooling, and development workflow.

---

## Checklist

### Repository

* [x] Create Git repository
* [x] Setup `.gitignore`
* [x] Add `README.md`
* [x] Add license

---

### Rust Project

* [x] Initialize cargo workspace
* [x] Create crates/modules:

  * [x] `cli`
  * [x] `tui`
  * [x] `db`
  * [x] `search`
  * [x] `session`
  * [x] `compare`
  * [x] `models`

---

### Dependencies

* [x] Add `clap`
* [x] Add `ratatui`
* [x] Add `crossterm`
* [x] Add `rusqlite`
* [x] Add `serde`
* [x] Add `serde_json`
* [x] Add `fuzzy-matcher`
* [x] Add `directories`
* [x] Add `tokio` only if required

---

### File Structure

* [x] Setup:

```text
src/
data/
config/
tests/
```

---

### Config Paths

* [x] Linux config path:

```text
~/.config/codelex/
```

* [x] Linux data path:

```text
~/.local/share/codelex/
```

---

### CI

* [x] Add formatting check
* [x] Add clippy check
* [x] Add build check

---

# Phase 1. Core Data Model

## Goal

Design deterministic syntax storage system.

---

## Checklist

### SQLite Schema

* [x] Create `languages`
* [x] Create `topics`
* [x] Create `snippets`
* [x] Create `aliases`
* [x] Create `related_topics`
* [x] Create `sessions`
* [x] Create `session_queries`

---

### Rust Models

* [x] Create `Language`
* [x] Create `Topic`
* [x] Create `Snippet`
* [x] Create `Alias`
* [x] Create `Session`

---

### Database Layer

* [x] SQLite connection manager
* [x] Query helper functions
* [x] Migration system
* [x] Seed loader

---

### Seed Data Format

* [x] Define JSON schema
* [x] Create parser
* [x] Create validation logic

---

### Initial Data

* [x] Python snippets
* [x] Rust snippets
* [x] Go snippets
* [x] JS snippets
* [x] TypeScript snippets

---

### Initial Concepts

Minimum:

* [x] arrays
* [x] hashmap
* [x] iteration
* [x] sorting
* [x] files
* [x] json
* [x] async
* [x] strings
* [x] regex

---

# Phase 2. Stateless CLI

## Goal

Create ultra-fast syntax lookup.

---

## Checklist

### CLI Parser

* [x] Setup clap commands
* [x] Add language argument
* [x] Add query parsing
* [x] Add flags:

  * [x] `--more`
  * [x] `--raw`
  * [x] `--json`

---

### Query Engine

* [x] Normalize query
* [x] Alias expansion
* [x] Exact match search
* [x] Partial match search
* [x] Fuzzy search

---

### Output Rendering

* [x] Minimal output mode
* [x] Expanded output mode
* [x] Syntax highlighting
* [x] Related topics rendering

---

### Compare Command

* [x] Add compare parser
* [x] Multi-language retrieval
* [x] Side-by-side formatting

---

### Error Handling

* [x] Unknown language
* [x] Empty result
* [x] Ambiguous result

---

### Performance

* [x] Query benchmark
* [x] Cold start benchmark
* [x] Optimize DB indexes

---

# Phase 3. Search System

## Goal

Build fast, intelligent retrieval.

---

## Checklist

### Search Normalization

* [x] Lowercasing
* [x] Symbol cleanup
* [x] Token splitting

---

### Alias System

* [x] Alias table integration
* [x] Synonym expansion
* [x] Cross-language alias mapping

---

### Ranking Engine

* [x] Exact match weighting
* [x] Alias weighting
* [x] Language weighting
* [x] Usage weighting

---

### Fuzzy Search

* [x] Implement fuzzy matcher
* [x] Tune scoring
* [x] Benchmark large datasets

---

### Related Concepts

* [x] Graph lookup
* [x] Related suggestions
* [x] Similar concept ranking

---

# Phase 4. Session System

## Goal

Introduce contextual language memory.

---

## Checklist

### Session Creation

* [x] Create session command
* [x] Persist session metadata
* [x] Set active language

---

### Session Storage

* [x] Save query history
* [x] Save timestamps
* [x] Save bookmarks

---

### Session Commands

* [x] `session create`
* [x] `session list`
* [x] `session delete`
* [x] `session switch`

---

### Session Context Logic

* [x] Auto-language prioritization
* [x] Session-aware ranking
* [x] Recent query weighting

---

### Session Persistence

* [x] Restore previous session
* [x] Handle corruption safely

---

# Phase 5. TUI Foundation

## Goal

Build terminal-native interface.

---

## Checklist

### Terminal Setup

* [x] Raw mode
* [x] Alternate screen
* [x] Event loop

---

### Layout System

* [x] Header panel
* [x] Query panel
* [x] Result panel
* [x] Related panel
* [x] History panel

---

### Keyboard Navigation

* [x] Vim-style movement
* [x] Tab switching
* [x] Search submission
* [x] Quit handling

---

### Query Input

* [x] Typing support
* [x] Backspace
* [x] Cursor movement

---

### Result Rendering

* [x] Syntax formatting
* [x] Line wrapping
* [x] Scroll support

---

### Theme System

* [x] Default theme
* [x] Border styling
* [x] Syntax colors

---

# Phase 6. Advanced TUI Features

## Goal

Make TUI genuinely useful daily.

---

## Checklist

### History System

* [x] Recent queries
* [x] History navigation
* [x] Persistent history

---

### Bookmarks

* [x] Bookmark snippets
* [x] Bookmark browser
* [x] Delete bookmarks

---

### Pinning

* [x] Pin common snippets
* [x] Session pins

---

### Compare Mode

* [x] Inline compare rendering
* [x] Multi-column rendering
* [x] Language selector

---

### Quick Navigation

* [x] Fuzzy topic picker
* [ ] Jump-to-category

---

### Related Navigation

* [ ] Navigate related topics
* [x] Open related topic instantly

---

# Phase 7. Snippet Expansion

## Goal

Increase usefulness breadth.

---

## Checklist

### Add Languages

* [x] C++
* [x] Bash
* [x] Java
* [x] Kotlin

---

### Expand Concepts

* [x] networking
* [x] testing
* [x] concurrency
* [x] serialization
* [x] environment variables
* [x] CLI parsing
* [x] filesystem ops

---

### Improve Snippet Quality

* [x] Standardize formatting
* [x] Reduce verbosity
* [x] Add idiomatic examples

---

### Validation

* [ ] Compile Rust snippets
* [ ] Run Python snippets
* [x] Validate syntax correctness

---

# Phase 8. Polish

## Goal

Make product feel production-grade.

---

## Checklist

### Startup Experience

* [x] Welcome screen
* [x] First-run setup
* [x] Default session generation

---

### Config

* [x] Config loader
* [x] Theme config
* [x] Language defaults

---

### Help System

* [x] CLI help
* [x] TUI shortcuts page
* [x] Command examples

---

### Logging

* [x] Debug logs
* [x] Error logs

---

### Packaging

* [x] Cargo install support
* [x] Arch PKGBUILD
* [x] Binary releases

---

### README

* [ ] GIF demos
* [x] Installation guide
* [x] Usage examples
* [x] Architecture overview

---

# Phase 9. Optional Extensions

## Goal

Post-MVP improvements only.

---

## Checklist

### Neovim Integration

* [x] Floating window
* [x] Inline search
* [ ] Session sync

---

### Shell Integration

* [x] Bash integration
* [x] Zsh integration
* [x] Fish integration

---

### Snippet Packs

* [x] Community packs
* [ ] Import/export

---

### Analytics

LOCAL ONLY.

* [x] Most used snippets
* [x] Session stats

---

### Offline Docs

* [x] Tiny doc bundles
* [x] Curated references

---

# Development Priority Order

## Mandatory MVP

1. Core DB
2. CLI
3. Search
4. Sessions
5. Basic TUI

---

## High Value

6. Compare mode
7. Related concepts
8. History

---

## Nice-to-have

9. Bookmarks
10. Themes
11. Neovim integration

---

# Final MVP Definition

Project is MVP-complete when:

* [x] User can retrieve syntax in under 2 seconds
* [x] Sessions persist correctly
* [x] Compare mode works
* [x] TUI is stable
* [x] Snippets are accurate
* [x] Query latency feels instant
* [x] Product works fully offline
* [x] No AI required
* [x] Tool genuinely replaces browser lookup for common syntax recall tasks
