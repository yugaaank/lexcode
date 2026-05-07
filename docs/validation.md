# Validation

Run:

```bash
cargo run -- validate
```

The validator checks:

- built-in JSON parses correctly
- the in-memory SQLite database migrates and seeds
- all snippets remain short enough for one-screen recall
- supplemental languages are present in the effective database

The snippets are intentionally fragments, so full compilation is only practical for future language-specific harnesses.
