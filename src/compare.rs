use crate::db::Database;
use crate::models::SearchResult;
use crate::search;

const DEFAULT_COMPARE_LANGUAGES: &[&str] = &[
    "python",
    "rust",
    "go",
    "javascript",
    "typescript",
    "c++",
    "bash",
    "java",
    "kotlin",
];

pub fn compare(
    database: &Database,
    query: &str,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    for language in DEFAULT_COMPARE_LANGUAGES {
        if let Some(result) = search::search(database, Some(language), query)?
            .into_iter()
            .find(|result| result.language == *language)
        {
            output.push(result);
        }
    }
    Ok(output)
}

pub fn render(
    results: &[SearchResult],
    raw: bool,
    json: bool,
) -> Result<String, serde_json::Error> {
    if json {
        return serde_json::to_string_pretty(results);
    }
    if raw {
        return Ok(results
            .iter()
            .map(|result| result.snippets.first().cloned().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n"));
    }

    Ok(render_columns(results))
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn render_columns(results: &[SearchResult]) -> String {
    const WIDTH: usize = 34;
    const PER_ROW: usize = 3;
    let blocks = results
        .iter()
        .map(|result| {
            let mut lines = vec![title_case(&result.language)];
            lines.extend(
                result
                    .snippets
                    .first()
                    .cloned()
                    .unwrap_or_default()
                    .lines()
                    .map(ToOwned::to_owned),
            );
            lines
        })
        .collect::<Vec<_>>();

    let mut output = Vec::new();
    for row in blocks.chunks(PER_ROW) {
        let height = row.iter().map(Vec::len).max().unwrap_or(0);
        for line_index in 0..height {
            let line = row
                .iter()
                .map(|block| {
                    let text = block.get(line_index).map(String::as_str).unwrap_or("");
                    let clipped = if text.len() > WIDTH {
                        &text[..WIDTH]
                    } else {
                        text
                    };
                    format!("{clipped:<WIDTH$}")
                })
                .collect::<Vec<_>>()
                .join("  ");
            output.push(line.trim_end().to_string());
        }
        output.push(String::new());
    }
    while output.last().is_some_and(String::is_empty) {
        output.pop();
    }
    output.join("\n")
}
