# ⚡️ Codelex

> **Syntax recall at the speed of thought.** Stop paying the context-switch tax.

```text
    _           _      _            
   | |         | |    | |           
   | |     ___ | |  __| | ___ __  __
   | |    / _ \| | / _` |/ _ \\ \/ /
   | |___|  __/| || (_| |  __/ >  < 
   \_____/\___||_| \__,_|\___|/_/\_\
                                    
```

**Codelex** is a terminal-native syntax recall system for polyglot developers who are tired of SEO garbage, StackOverflow archaeology, and AI hallucinations. 

It doesn't "generate" code. It **retrieves** curated, idiomatic snippets from a local SQLite database faster than you can Alt-Tab.

---

## 🧠 The Problem: The "Context Switch Tax"

You're deep in a Rust backend, and suddenly you need to write a quick Python script to process some JSON. 

*Is it `.push()` or `.append()`?*
*Wait, how do I sort a list again?*
*Is it `json.loads()` or `json.parse()`?*

Your brain stalls. You open a browser. You get lost in a 2014 StackOverflow thread. **Flow state: Terminated.**

---

## 🚀 The Solution: Codelex

Codelex keeps you in the zone. It’s like a second brain for your terminal.

- **Offline First:** No API keys, no latency, no "as an AI model...".
- **Zero Hallucinations:** Every snippet is curated and verified.
- **Blazing Fast:** <100ms lookup. It's basically muscle memory.
- **Polyglot Power:** Compare syntax across languages in a single view.

---

## 🕹 Quick Start

Get it running before your next coffee:

```bash
# Clone and build
git clone https://github.com/youruser/lexcode.git
cd lexcode
cargo install --path .

# Grab a snippet
codelex rs vec sort
```

---

## 🛠 Commands for Power Users

| Command | What it does |
| :--- | :--- |
| `codelex <lang> <query>` | The classic. Instant syntax. |
| `codelex compare <query>` | See how different languages handle the same concept. |
| `codelex session <cmd>` | Manage persistent context (bookmarks, history, pins). |
| `codelex tui` | Launch the beautiful Ratatui interface. |

### Examples:
```bash
# "How do I do X in Y?"
codelex py dict iterate
codelex rs file read

# "Wait, how does Go do what I just did in Rust?"
codelex compare hashmap insert

# "I'm working on a CLI project today."
codelex session create cli-work --language rust
```

---

## 📺 The TUI (The Main Event)

Type `codelex tui` and enter the matrix. 

- **Vim-style navigation** (`j`/`k`).
- **Instant fuzzy search** as you type.
- **Pin & Bookmark** your most frequent "I forgot this again" snippets.
- **Themeable** because aesthetics matter.

---

## 🏗 Why Codelex?

We believe **Syntax > Explanation**. You don't need a tutorial; you need the exact 3 lines of code that solve your problem right now.

Codelex is built with 🦀 **Rust** and **SQLite** for maximum performance and reliability. It follows the philosophy of tools like `ripgrep`, `fzf`, and `git`: do one thing, do it fast, and stay out of the way.

---

## 🤝 Contributing

Got a better way to do something in C++? Found a missing Go snippet?
Check out `docs/snippet-packs.md` and help us make the ultimate syntax reference!

---

<p align="center">
  Made with ❤️ for the terminal-obsessed.
</p>
