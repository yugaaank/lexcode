use std::io::{self, IsTerminal};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

use crate::compare;
use crate::config::Config;
use crate::db::Database;
use crate::models::SearchResult;
use crate::{search, session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    Query,
    Result,
    Related,
    History,
}

pub fn run(database: &Database, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if !io::stdout().is_terminal() {
        println!("Codelex TUI requires an interactive terminal.");
        return Ok(());
    }

    let active = session::ensure_default(database, &config.default_language)?;
    let languages = database.list_languages()?;
    let mut app = App::new(active.language.clone(), active.name.clone(), languages);
    app.search(database)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_loop(database, &mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_loop(
    database: &Database,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if !event::poll(Duration::from_millis(120))? {
            continue;
        }
        if let Event::Key(key) = event::read()?
            && handle_key(database, app, key)?
        {
            break;
        }
    }
    Ok(())
}

fn handle_key(
    database: &Database,
    app: &mut App,
    key: KeyEvent,
) -> Result<bool, Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('q') if key.modifiers.is_empty() => return Ok(true),
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => return Ok(true),
        KeyCode::Tab => app.next_panel(),
        KeyCode::Char('/') => app.panel = Panel::Query,
        KeyCode::Char('j') | KeyCode::Down => app.scroll = app.scroll.saturating_add(1),
        KeyCode::Char('k') | KeyCode::Up => app.scroll = app.scroll.saturating_sub(1),
        KeyCode::Char('c') => {
            app.compare_mode = !app.compare_mode;
            app.search(database)?;
        }
        KeyCode::Char('l') => {
            app.next_language();
            app.search(database)?;
        }
        KeyCode::Char('b') => {
            if let Some(result) = &app.result {
                session::bookmark(database, &app.session_name, result)?;
                app.message = "bookmarked".to_string();
            }
        }
        KeyCode::Char('p') => {
            if let Some(result) = &app.result {
                session::pin(database, &app.session_name, result)?;
                app.message = "pinned".to_string();
            }
        }
        KeyCode::Enter => {
            if app.panel == Panel::Related {
                if let Some(topic) = app
                    .result
                    .as_ref()
                    .and_then(|result| result.related.first())
                    .cloned()
                {
                    app.query = topic;
                    app.cursor = app.query.len();
                }
            } else if app.panel == Panel::History
                && let Some(query) = app.history.first().cloned()
            {
                app.query = query;
                app.cursor = app.query.len();
            }
            app.search(database)?;
        }
        KeyCode::Backspace if app.panel == Panel::Query => {
            app.query.pop();
            app.search(database)?;
        }
        KeyCode::Left if app.panel == Panel::Query => app.cursor = app.cursor.saturating_sub(1),
        KeyCode::Right if app.panel == Panel::Query => {
            app.cursor = (app.cursor + 1).min(app.query.len());
        }
        KeyCode::Char(character) if app.panel == Panel::Query => {
            app.query.insert(app.cursor.min(app.query.len()), character);
            app.cursor += 1;
            app.search(database)?;
        }
        _ => {}
    }
    Ok(false)
}

fn draw(frame: &mut ratatui::Frame<'_>, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
            Constraint::Length(5),
        ])
        .split(area);

    let header = Paragraph::new(format!(
        "Codelex  Session: {}  Language: {}  Mode: {}  {}",
        app.session_name,
        app.language,
        if app.compare_mode {
            "compare"
        } else {
            "lookup"
        },
        app.message
    ))
    .block(Block::default().borders(Borders::ALL).title("Header"));
    frame.render_widget(header, chunks[0]);

    let query = Paragraph::new(format!("> {}", app.query))
        .style(if app.panel == Panel::Query {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        })
        .block(Block::default().borders(Borders::ALL).title("Query"));
    frame.render_widget(query, chunks[1]);

    let result_text = if app.compare_mode {
        app.compare_text.clone()
    } else {
        app.result
            .as_ref()
            .map(render_result)
            .unwrap_or_else(|| "No result".to_string())
    };
    let result = Paragraph::new(result_text)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0))
        .block(Block::default().borders(Borders::ALL).title("Result"));
    frame.render_widget(result, chunks[2]);

    let related = app
        .result
        .as_ref()
        .map(|result| result.related.clone())
        .unwrap_or_default()
        .into_iter()
        .map(ListItem::new)
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(related).block(Block::default().borders(Borders::ALL).title("Related")),
        chunks[3],
    );

    let history = app
        .history
        .iter()
        .map(|item| {
            ListItem::new(Line::from(vec![
                Span::styled(">", Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::raw(item.clone()),
            ]))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(history).block(Block::default().borders(Borders::ALL).title("History")),
        chunks[4],
    );
}

fn render_result(result: &SearchResult) -> String {
    let mut output = vec![
        format!("Topic: {}", result.topic),
        format!("Language: {}", result.language),
        String::new(),
        result.snippets.join("\n"),
    ];
    if !result.related.is_empty() {
        output.push(String::new());
        output.push("Related:".to_string());
        output.extend(result.related.iter().map(|item| format!("- {item}")));
    }
    output.join("\n")
}

#[derive(Debug)]
struct App {
    language: String,
    session_name: String,
    query: String,
    cursor: usize,
    result: Option<SearchResult>,
    compare_text: String,
    history: Vec<String>,
    panel: Panel,
    scroll: u16,
    compare_mode: bool,
    languages: Vec<String>,
    message: String,
}

impl App {
    fn new(language: String, session_name: String, languages: Vec<String>) -> Self {
        Self {
            language,
            session_name,
            query: "hashmap iterate".to_string(),
            cursor: "hashmap iterate".len(),
            result: None,
            compare_text: String::new(),
            history: Vec::new(),
            panel: Panel::Query,
            scroll: 0,
            compare_mode: false,
            languages,
            message:
                "Tab panels  Enter open/search  l language  b bookmark  p pin  c compare  q quit"
                    .to_string(),
        }
    }

    fn search(&mut self, database: &Database) -> Result<(), Box<dyn std::error::Error>> {
        self.scroll = 0;
        let history = session::history(database, &self.session_name, 8)?;
        self.history = history.into_iter().map(|item| item.query).collect();
        if self.query.trim().is_empty() {
            self.result = None;
            return Ok(());
        }

        if self.compare_mode {
            let results = compare::compare(database, &self.query)?;
            self.compare_text = compare::render(&results, false, false)?;
            self.result = results.first().cloned();
        } else {
            let results = search::search(database, Some(&self.language), &self.query)?;
            self.result = results
                .into_iter()
                .find(|result| result.language == self.language);
        }
        session::record_query(database, &self.session_name, &self.query)?;
        Ok(())
    }

    fn next_panel(&mut self) {
        self.panel = match self.panel {
            Panel::Query => Panel::Result,
            Panel::Result => Panel::Related,
            Panel::Related => Panel::History,
            Panel::History => Panel::Query,
        };
    }

    fn next_language(&mut self) {
        if self.languages.is_empty() {
            return;
        }
        let index = self
            .languages
            .iter()
            .position(|language| language == &self.language)
            .unwrap_or(0);
        self.language = self.languages[(index + 1) % self.languages.len()].clone();
        self.message = format!("language {}", self.language);
    }
}
