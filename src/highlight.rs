use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use crossterm::style::Stylize as CrosstermStylize;

pub fn highlight_tui(language: &str, snippet: &str) -> Vec<Line<'static>> {
    let keywords = get_keywords(language);
    let mut lines = Vec::new();

    for line_text in snippet.lines() {
        let mut spans = Vec::new();
        let mut token = String::new();

        for character in line_text.chars() {
            if character.is_ascii_alphanumeric() || character == '_' {
                token.push(character);
                continue;
            }
            
            if !token.is_empty() {
                spans.push(style_token_tui(&token, keywords));
                token.clear();
            }
            spans.push(Span::raw(character.to_string()));
        }
        
        if !token.is_empty() {
            spans.push(style_token_tui(&token, keywords));
        }
        
        lines.push(Line::from(spans));
    }
    
    lines
}

pub fn highlight_cli(language: &str, snippet: &str) -> String {
    let keywords = get_keywords(language);
    let mut output = String::new();
    let mut token = String::new();

    for character in snippet.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            token.push(character);
            continue;
        }
        
        if !token.is_empty() {
            output.push_str(&style_token_cli(&token, keywords));
            token.clear();
        }
        output.push(character);
    }
    
    if !token.is_empty() {
        output.push_str(&style_token_cli(&token, keywords));
    }
    
    output
}

fn get_keywords(language: &str) -> &'static [&'static str] {
    match language {
        "rust" => &["let", "for", "in", "async", "fn", "return", "mut", "pub", "use", "mod", "struct", "enum", "impl", "match", "if", "else"][..],
        "python" => &["from", "import", "for", "in", "async", "def", "return", "if", "elif", "else", "with", "as", "class", "try", "except"],
        "go" => &["for", "range", "func", "go", "var", "return", "if", "else", "struct", "interface", "package", "import"],
        "javascript" | "typescript" => {
            &["const", "let", "for", "of", "async", "function", "return", "if", "else", "export", "import", "class", "await"]
        }
        _ => &[][..],
    }
}

fn style_token_tui(token: &str, keywords: &[&str]) -> Span<'static> {
    if keywords.contains(&token) {
        Span::styled(token.to_string(), Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD))
    } else if token.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        Span::styled(token.to_string(), Style::default().fg(Color::Yellow))
    } else {
        Span::raw(token.to_string())
    }
}

fn style_token_cli(token: &str, keywords: &[&str]) -> String {
    if keywords.contains(&token) {
        CrosstermStylize::cyan(token).bold().to_string()
    } else if token.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        CrosstermStylize::yellow(token).to_string()
    } else {
        token.to_string()
    }
}
