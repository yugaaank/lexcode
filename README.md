# Codelex

Codelex is a terminal-native syntax recall system for developers who switch languages often.

It is not a chatbot and does not generate code. It retrieves curated snippets from a local SQLite database.

## Usage

```bash
codelex rs vec sort
codelex py json load --more
codelex compare hashmap insert
codelex session create rust-api --language rust
codelex session switch rust-api
codelex tui
```

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -- rs hashmap iterate
cargo run -- bench "hashmap iterate"
cargo run -- validate
```

## Storage

Codelex uses platform data directories:

- Config: `~/.config/codelex/`
- Data: `~/.local/share/codelex/`

On first run it creates and seeds `codelex.db` in the data directory.

## Commands

- `codelex <language> <query>`: retrieve one syntax answer.
- `codelex compare <query>`: compare equivalent syntax across languages.
- `codelex session create|list|delete|switch|active|history|bookmarks|pins|stats`: manage persistent context.
- `codelex tui`: launch the terminal UI.
- `codelex bench`: print cold-start and query latency.
- `codelex validate`: validate built-in snippets.

## TUI Keys

- `Tab`: switch panel
- `Enter`: search
- `/`: focus query
- `j/k` or arrows: scroll
- `b`: bookmark current result
- `p`: pin current result
- `c`: toggle compare mode
- `l`: cycle language
- `q`: quit

## Architecture

- `src/db.rs`: SQLite schema, migrations, seed loading, indexes.
- `src/search.rs`: normalization, alias matching, ranking, fuzzy matching.
- `src/session.rs`: active session, history, bookmarks, pins.
- `src/tui.rs`: Ratatui interface.
- `data/snippets.json`: curated offline syntax pack.
