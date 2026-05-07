use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct Frontmatter {
    pub topic: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub related: Vec<String>,
}

#[derive(Debug)]
pub struct PackSnippet {
    pub language: String,
    pub frontmatter: Frontmatter,
    pub snippet: String,
}

pub fn parse_pack(packs_dir: &Path) -> Result<Vec<PackSnippet>, Box<dyn std::error::Error>> {
    let mut snippets = Vec::new();

    if !packs_dir.exists() {
        return Ok(snippets);
    }

    for entry in WalkDir::new(packs_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(snippet) = parse_markdown_file(path)? {
                snippets.push(snippet);
            }
        }
    }

    Ok(snippets)
}

fn parse_markdown_file(path: &Path) -> Result<Option<PackSnippet>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    
    // Simple frontmatter parser: expects files to start with ---
    if !content.starts_with("---") {
        return Ok(None);
    }

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Ok(None);
    }

    let frontmatter_str = parts[1];
    let body = parts[2];

    let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter_str)?;
    
    // Extract code block from body
    // Expects: ```lang\ncode\n```
    let mut snippet = String::new();
    let mut in_code_block = false;

    for line in body.lines() {
        if line.starts_with("```") {
            if in_code_block {
                break;
            } else {
                in_code_block = true;
                continue;
            }
        }
        if in_code_block {
            if !snippet.is_empty() {
                snippet.push('\n');
            }
            snippet.push_str(line);
        }
    }

    if snippet.is_empty() {
        return Ok(None);
    }

    // Determine language from directory structure: packs/<language>/file.md
    let language = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(Some(PackSnippet {
        language,
        frontmatter,
        snippet,
    }))
}
