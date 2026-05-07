
# Codelex

## Product Definition

A terminal-native syntax recall system for developers switching programming languages frequently.

Not documentation.
Not AI chat.
Not tutorial software.

Primary job:
Restore coding momentum after context switching.

Core problem:
Developers remember concepts but forget implementation shape.

Example:

* “How do I sort again in Rust?”
* “What’s Python’s JSON load syntax?”
* “How do I iterate HashMap in Go?”

Browser search destroys flow:

* tab switching
* SEO garbage
* StackOverflow archaeology
* context loss

Codelex compresses retrieval latency to near muscle memory.

---

# Product Identity

## Emotional Positioning

Should feel like:

* `git`
* `ripgrep`
* `fzf`
* `htop`
* `btop`
* `man`

NOT:

* chatbot
* assistant
* copilot
* SaaS dashboard

Tone:

* utilitarian
* sharp
* low-level
* fast
* quiet

---

# User Archetype

## Primary User

Polyglot developers.

People switching between:

* Rust
* Python
* Go
* JS/TS
* C++
* Bash

Examples:

* backend engineers
* systems programmers
* open source contributors
* Linux users
* competitive programmers
* DevOps engineers

---

# Core Use Cases

## Use Case 1

### Context-switch syntax loss

Morning:

```text id="6wkj4y"
Python ML work
```

Night:

```text id="ywkvw5"
Rust backend
```

Brain conflict:

```text id="xlnyjs"
append?
push?
insert?
extend?
```

Codelex resolves instantly.

---

## Use Case 2

### Interrupt recovery

User forgot:

```text id="sj8q8k"
Rust serde JSON parsing syntax
```

Instead of:

* browser
* docs
* StackOverflow

User:

```bash id="9k8yo4"
codelex serde json
```

Done.

---

## Use Case 3

### Cross-language mapping

User knows concept in Python.
Needs Rust equivalent.

Example:

```bash id="e29wth"
codelex compare file read
```

---

# Core Product Philosophy

## Principle 1

### Syntax > Explanation

Bad:

```text id="dgr7rb"
“In Rust, vectors can be sorted using...”
```

Good:

```rust id="iwx5wk"
vec.sort();
```

---

## Principle 2

### One-screen retrieval

No scrolling through tutorials.

---

## Principle 3

### Instantaneous

Target:

```text id="7jn2kb"
<100ms perceived latency
```

---

## Principle 4

### Keyboard-only workflow

Mouse usage should feel wrong.

---

## Principle 5

### Deterministic output

No hallucination.
No generated code.

Everything curated.

---

# Product Surface

# 1. CLI MODE

## Purpose

Fast stateless retrieval.

---

## Command Structure

```bash id="p2oqns"
codelex <language> <query>
```

Examples:

```bash id="sudlfe"
codelex py sort list
codelex rs hashmap iterate
codelex go read file
codelex cpp unordered map
```

---

## Smart Aliases

```bash id="jrfq8w"
py  -> python
rs  -> rust
js  -> javascript
ts  -> typescript
cpp -> c++
```

---

## Output Design

### Minimal Mode

```bash id="fy4q6n"
$ codelex rs vec sort
```

Output:

```rust id="qj4v47"
vec.sort();
vec.sort_by(|a, b| a.cmp(b));
```

---

## Expanded Mode

```bash id="4s5pjc"
$ codelex rs vec sort --more
```

Output:

```text id="x5i2we"
Topic: sort vector
Language: Rust

vec.sort();
vec.sort_unstable();

Related:
- binary_search
- reverse
- sort_by
```

---

# 2. SESSION MODE

## Purpose

Persistent language cognition.

This is the real innovation.

---

## Session Structure

```bash id="7mn5ya"
codelex session rust-api
```

Creates:

```text id="vcdcm8"
~/.local/share/codelex/sessions/rust-api.db
```

Tracks:

* active language
* query history
* pinned snippets
* bookmarks
* recent concepts

---

## Session State

```json id="nwxn93"
{
  "name": "rust-api",
  "language": "rust",
  "recent_queries": [
    "serde json",
    "tokio mutex",
    "vec iterate"
  ]
}
```

---

## Session Commands

```bash id="6i7g6g"
codelex session rust-api
codelex session list
codelex session delete rust-api
codelex session switch rust-api
```

---

# 3. TUI MODE

## Launch

```bash id="22gtzh"
codelex tui
```

or:

```bash id="k6f7n4"
codelex
```

---

# TUI DESIGN

# Layout

```text id="8kek0o"
┌─────────────────────────────────────┐
│ Codelex                            │
│ Session: rust-api                  │
│ Language: Rust                     │
├─────────────────────────────────────┤
│ Query                              │
│ > hashmap iterate                  │
├─────────────────────────────────────┤
│ Result                             │
│ for (k, v) in &map {}              │
│                                     │
│ map.iter()                         │
├─────────────────────────────────────┤
│ Related                            │
│ vec iterate                        │
│ btreemap                           │
│ hashmap insert                     │
├─────────────────────────────────────┤
│ History                            │
│ serde json                         │
│ async mutex                        │
│ vec sort                           │
└─────────────────────────────────────┘
```

---

# Navigation

## Keys

```text id="ndh2sx"
Ctrl+j/k     move
Tab          switch panel
Enter        search
/            focus query
b            bookmark
p            pin
c            compare mode
q            quit
```

---

# Query Flow

## Step 1

User types:

```text id="jlwmkv"
hashmap iterate
```

---

## Step 2

Query resolver:

* normalize
* fuzzy match
* alias expansion

---

## Step 3

Result panel updates instantly.

---

# Compare Mode

## Trigger

```text id="k9g4e9"
:c hashmap insert
```

or:

```bash id="jmd8qo"
codelex compare hashmap insert
```

---

## Output

```text id="1tvt6n"
Python
d[key] = value

Rust
map.insert(key, value);

Go
m[key] = value

JavaScript
obj[key] = value
```

---

# DATA MODEL

# Core Entities

## Language

```sql id="7znh4g"
CREATE TABLE languages (
  id INTEGER PRIMARY KEY,
  name TEXT UNIQUE,
  alias TEXT
);
```

---

## Topics

```sql id="0apvaz"
CREATE TABLE topics (
  id INTEGER PRIMARY KEY,
  language_id INTEGER,
  title TEXT,
  category TEXT
);
```

---

## Snippets

```sql id="uxx9po"
CREATE TABLE snippets (
  id INTEGER PRIMARY KEY,
  topic_id INTEGER,
  snippet TEXT,
  explanation TEXT,
  priority INTEGER
);
```

---

## Aliases

```sql id="0dudof"
CREATE TABLE aliases (
  id INTEGER PRIMARY KEY,
  topic_id INTEGER,
  alias TEXT
);
```

---

## Related Topics

```sql id="4iv2m9"
CREATE TABLE related_topics (
  topic_id INTEGER,
  related_topic_id INTEGER
);
```

---

## Sessions

```sql id="g9utbz"
CREATE TABLE sessions (
  id INTEGER PRIMARY KEY,
  name TEXT UNIQUE,
  language TEXT,
  created_at DATETIME
);
```

---

## Session Queries

```sql id="9wdl1f"
CREATE TABLE session_queries (
  session_id INTEGER,
  query TEXT,
  timestamp DATETIME
);
```

---

# Search Engine

# Search Pipeline

## Step 1

Normalize:

```text id="j2gcmn"
HashMap Iterate
→
hashmap iterate
```

---

## Step 2

Alias expansion:

```text id="rnibn8"
dict → hashmap
```

---

## Step 3

Fuzzy scoring:

* exact match
* alias match
* partial token match

---

## Step 4

Language prioritization:
Current session language boosts rank.

---

# Ranking Formula

```text id="aw5sxq"
score =
exact_match * 100 +
alias_match * 50 +
language_match * 40 +
recent_usage * 20
```

---

# Snippet Design Rules

## Rule 1

Maximum:

```text id="j0h4sm"
5 lines
```

---

## Rule 2

Prefer idiomatic syntax.

---

## Rule 3

No tutorial explanations.

---

## Rule 4

Must solve:
“copy into code immediately.”

---

# Supported Categories

## Collections

* arrays
* vectors
* maps
* sets

---

## Iteration

* loops
* iterators
* enumerate

---

## Strings

* split
* join
* format

---

## Files

* read
* write
* append

---

## JSON

* parse
* serialize

---

## Async

* async function
* mutex
* channels

---

## HTTP

* request
* server

---

## Regex

* match
* capture

---

## Sorting

* sort
* custom sort

---

# Initial Language Support

## Phase 1

* Rust
* Python
* Go
* JavaScript
* TypeScript

---

## Phase 2

* C++
* Bash
* Java
* Kotlin

---

# Snippet Curation Philosophy

Do NOT scrape StackOverflow.

Curate manually.

Reason:

* syntax consistency
* quality
* idiomatic correctness
* formatting coherence

---

# File Structure

```text id="uqk6rf"
src/
  cli/
  tui/
  search/
  db/
  sessions/
  compare/

data/
  rust.json
  python.json
  go.json

config/
```

---

# Config System

```toml id="i8dbcl"
default_language = "rust"
theme = "dark"
max_history = 100
show_related = true
```

Path:

```text id="lqyvfc"
~/.config/codelex/config.toml
```

---

# Startup Flow

```text id="sl2khn"
Load config
→
Load sqlite DB
→
Load session state
→
Initialize fuzzy index
→
Render TUI
```

---

# Performance Targets

## Cold Start

```text id="m8zv9l"
<150ms
```

---

## Query Resolution

```text id="mqg63y"
<50ms
```

---

## Memory

```text id="cvy8pb"
<100MB
```

---

# Visual Design

## Colors

Minimal.

Inspired by:

* btop
* lazygit
* helix

No gradients.
No glassmorphism nonsense.

---

## Typography

Monospace only.

---

## Borders

Thin.
Muted.

---

## Syntax Highlighting

Only:

* keywords
* types
* strings

No rainbow vomit.

---

# Future Expansion

ONLY after core succeeds.

---

## Neovim Plugin

```vim id="i5u8w8"
:Codelex hashmap
```

Floating window.

---

## Shell Integration

Example:

```bash id="g9z3r3"
Ctrl+h
```

Opens quick syntax popup.

---

## Offline Doc Packs

Tiny curated packs.

NOT full docs.

---

## Team Snippet Packs

Example:

```text id="2r4avf"
company-rust-pack
```

Internal patterns.

---

# What Makes This Interesting

Not the snippets.

Not the TUI.

The insight:
Programming context switching has cognitive overhead.

Codelex minimizes:
syntax reconstruction latency.

That is a real, narrow, technically believable product thesis.
