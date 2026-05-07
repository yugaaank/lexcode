# Snippet Packs

Codelex currently ships with a built-in curated pack loaded from `data/snippets.json` plus supplemental built-in topics.

Future community packs should use the same shape:

```json
{
  "languages": [
    {
      "name": "rust",
      "aliases": ["rs"],
      "topics": [
        {
          "title": "hashmap iterate",
          "category": "iteration",
          "aliases": ["map iterate"],
          "snippets": ["for (key, value) in &map {}"],
          "related": ["hashmap insert"]
        }
      ]
    }
  ]
}
```

Imports should be deterministic, idempotent, and local only.
