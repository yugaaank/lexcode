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
use crate::highlight;
use crate::models::SearchResult;
use crate::{search, session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    Query,
    Result,
    Related,
    History,
    Settings,
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
        if !event::poll(Duration::from_millis(100))? {
            app.tick = app.tick.wrapping_add(1);
            continue;
        }
        if let Event::Key(key) = event::read()?
            && handle_key(database, app, key)?
        {
            break;
        }
        app.tick = app.tick.wrapping_add(1);
    }
    Ok(())
}

fn handle_key(
    database: &Database,
    app: &mut App,
    key: KeyEvent,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Global Exit Keys
    if key.code == KeyCode::Char('q') && key.modifiers.is_empty() { return Ok(true); }
    if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL { return Ok(true); }

    // Panel Navigation
    if key.code == KeyCode::Tab {
        app.next_panel();
        return Ok(false);
    }

    match app.panel {
        Panel::Query => match key.code {
            KeyCode::Char(c) => {
                app.query.insert(app.cursor, c);
                app.cursor += 1;
                app.search(database)?;
            }
            KeyCode::Backspace => {
                if app.cursor > 0 {
                    app.query.remove(app.cursor - 1);
                    app.cursor -= 1;
                    app.search(database)?;
                }
            }
            KeyCode::Left => app.cursor = app.cursor.saturating_sub(1),
            KeyCode::Right => app.cursor = (app.cursor + 1).min(app.query.len()),
            KeyCode::Enter => {
                app.panel = Panel::Result;
                app.search(database)?;
            }
            _ => {}
        },
        Panel::Settings => match key.code {
            KeyCode::Char('l') => {
                app.next_language();
                app.search(database)?;
            }
            KeyCode::Char('c') => {
                app.compare_mode = !app.compare_mode;
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
            _ => {}
        },
        _ => match key.code {
            KeyCode::Char('j') | KeyCode::Down => app.scroll = app.scroll.saturating_add(1),
            KeyCode::Char('k') | KeyCode::Up => app.scroll = app.scroll.saturating_sub(1),
            KeyCode::Enter => {
                if app.panel == Panel::Related {
                    if let Some(topic) = app.result.as_ref().and_then(|r| r.related.first()).cloned() {
                        app.query = topic;
                        app.cursor = app.query.len();
                        app.search(database)?;
                    }
                } else if app.panel == Panel::History {
                    if let Some(query) = app.history.first().cloned() {
                        app.query = query;
                        app.cursor = app.query.len();
                        app.search(database)?;
                    }
                }
            }
            _ => {}
        },
    }
    Ok(false)
}

fn draw(frame: &mut ratatui::Frame<'_>, app: &App) {
    let area = frame.area();
    
    // Background style
    frame.render_widget(Block::default().style(Style::default().bg(Color::Reset)), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Query
            Constraint::Min(10),   // Main Content (Result + Sidebar)
            Constraint::Length(1), // Status Bar / Settings
        ])
        .split(area);

    // 1. Query Area (Spotlight style)
    let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spin_char = if app.searching { spinner[(app.tick as usize) % spinner.len()] } else { "›" };
    
    let query_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(if app.panel == Panel::Query {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(Span::styled(" Search Concept ", Style::default().fg(Color::Cyan)));
    
    let cursor_visible = (app.tick / 5) % 2 == 0;
    let mut query_spans = vec![
        Span::styled(format!(" {} ", spin_char), Style::default().fg(Color::Cyan)),
        Span::raw(&app.query[..app.cursor]),
    ];
    if app.panel == Panel::Query && cursor_visible {
        query_spans.push(Span::styled("█", Style::default().fg(Color::Cyan)));
    } else if app.panel == Panel::Query {
        query_spans.push(Span::raw(" "));
    }
    query_spans.push(Span::raw(&app.query[app.cursor..]));

    let query = Paragraph::new(Line::from(query_spans))
        .block(query_block);
    frame.render_widget(query, chunks[0]);

    // 2. Main Content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(75), // Result
            Constraint::Percentage(25), // Sidebar (Related + History)
        ])
        .split(chunks[1]);

    // Result Area
    let result_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(if app.panel == Panel::Result {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        })
        .title(Span::styled(format!(" {} Output ", app.language), Style::default().fg(Color::Yellow)));

    let result_content = if app.compare_mode {
        render_compare_results(app)
    } else if let Some(res) = &app.result {
        render_search_result(res)
    } else {
        vec![Line::from(Span::styled("Waiting for input...", Style::default().fg(Color::DarkGray)))]
    };

    let result = Paragraph::new(result_content)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0))
        .block(result_block);
    frame.render_widget(result, main_chunks[0]);

    // Sidebar
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Related
            Constraint::Percentage(50), // History
        ])
        .split(main_chunks[1]);

    // Related
    let related_items: Vec<ListItem> = app
        .result
        .as_ref()
        .map(|r| r.related.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|t| ListItem::new(format!(" • {}", t)).style(Style::default().fg(Color::Gray)))
        .collect();
    
    let related = List::new(related_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(if app.panel == Panel::Related { Style::default().fg(Color::Magenta) } else { Style::default().fg(Color::DarkGray) })
            .title(" Related "));
    frame.render_widget(related, sidebar_chunks[0]);

    // History
    let history_items: Vec<ListItem> = app.history
        .iter()
        .map(|h| ListItem::new(format!(" › {}", h)).style(Style::default().fg(Color::DarkGray)))
        .collect();

    let history = List::new(history_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(if app.panel == Panel::History { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) })
            .title(" History "));
    frame.render_widget(history, sidebar_chunks[1]);

    // 3. Settings Panel / Status Bar
    let settings_focused = app.panel == Panel::Settings;
    let status_style = if settings_focused {
        Style::default().bg(Color::White).fg(Color::Black)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };
    
    let status_content = Line::from(vec![
        Span::styled(" CODELEX ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(ratatui::style::Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(format!(" Session: {} ", app.session_name), if settings_focused { Style::default().fg(Color::Black) } else { Style::default().fg(Color::Gray) }),
        Span::raw(" | "),
        Span::styled(format!("(L) Lang: {} ", app.language), if settings_focused { Style::default().fg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD) } else { Style::default().fg(Color::White) }),
        Span::raw(" | "),
        Span::styled("(C) Compare ", if settings_focused && app.compare_mode { Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD) } else if settings_focused { Style::default().fg(Color::Black) } else { Style::default().fg(Color::White) }),
        Span::raw(" | "),
        Span::styled("(P) Pin ", if settings_focused { Style::default().fg(Color::Black) } else { Style::default().fg(Color::White) }),
        Span::raw(" | "),
        Span::styled(app.message.clone(), if settings_focused { Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD) } else { Style::default().fg(Color::Yellow) }),
    ]);
    let status_bar = Paragraph::new(status_content).style(status_style);
    frame.render_widget(status_bar, chunks[2]);
}

fn render_search_result(result: &SearchResult) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    
    lines.push(Line::from(vec![
        Span::styled(" Topic: ", Style::default().fg(Color::DarkGray)),
        Span::styled(result.topic.clone(), Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    for snippet in &result.snippets {
        let highlighted = highlight::highlight_tui(&result.language, snippet);
        for mut line in highlighted {
            // Add indentation to snippets
            line.spans.insert(0, Span::raw("  "));
            lines.push(line);
        }
        lines.push(Line::from(""));
    }

    lines
}

fn render_compare_results(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if app.compare_results.is_empty() {
        lines.push(Line::from(Span::styled("No comparison results", Style::default().fg(Color::DarkGray))));
        return lines;
    }

    for result in &app.compare_results {
        lines.push(Line::from(vec![
            Span::styled(" • ", Style::default().fg(Color::Cyan)),
            Span::styled(result.language.to_uppercase(), Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)),
        ]));
        
        for snippet in &result.snippets {
            let highlighted = highlight::highlight_tui(&result.language, snippet);
            for mut line in highlighted {
                line.spans.insert(0, Span::raw("   "));
                lines.push(line);
            }
        }
        lines.push(Line::from(""));
    }

    lines
}

#[derive(Debug)]
struct App {
    language: String,
    session_name: String,
    query: String,
    cursor: usize,
    result: Option<SearchResult>,
    compare_results: Vec<SearchResult>,
    history: Vec<String>,
    panel: Panel,
    scroll: u16,
    compare_mode: bool,
    languages: Vec<String>,
    message: String,
    searching: bool,
    tick: u64,
}

impl App {
    fn new(language: String, session_name: String, languages: Vec<String>) -> Self {
        Self {
            language,
            session_name,
            query: "hashmap iterate".to_string(),
            cursor: "hashmap iterate".len(),
            result: None,
            compare_results: Vec::new(),
            history: Vec::new(),
            panel: Panel::Query,
            scroll: 0,
            compare_mode: false,
            languages,
            message:
                "Tab to focus settings for (L)ang, (C)ompare, (B)ookmark, (P)in"
                    .to_string(),
            searching: false,
            tick: 0,
        }
    }

    fn search(&mut self, database: &Database) -> Result<(), Box<dyn std::error::Error>> {
        self.searching = true;
        self.scroll = 0;
        let history = session::history(database, &self.session_name, 8)?;
        self.history = history.into_iter().map(|item| item.query).collect();
        if self.query.trim().is_empty() {
            self.result = None;
            self.searching = false;
            return Ok(());
        }

        if self.compare_mode {
            self.compare_results = compare::compare(database, &self.query)?;
            self.result = self.compare_results.first().cloned();
        } else {
            let results = search::search(database, Some(&self.language), &self.query)?;
            self.result = results
                .into_iter()
                .find(|result| result.language == self.language);
        }
        session::record_query(database, &self.session_name, &self.query)?;
        self.searching = false;
        Ok(())
    }

    fn next_panel(&mut self) {
        self.panel = match self.panel {
            Panel::Query => Panel::Result,
            Panel::Result => Panel::Related,
            Panel::Related => Panel::History,
            Panel::History => Panel::Settings,
            Panel::Settings => Panel::Query,
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
