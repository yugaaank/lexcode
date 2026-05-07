use std::io::IsTerminal;
use std::time::Instant;

use clap::{Args, Parser, Subcommand};
use crossterm::style::Stylize;

use crate::compare;
use crate::config;
use crate::db::{Database, default_config_dir, default_data_dir};
use crate::highlight;
use crate::models::{SearchResult, SeedFile};
use crate::{search, session, tui};

#[derive(Debug, Parser)]
#[command(name = "codelex")]
#[command(about = "Terminal-native syntax recall.")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(help = "Language or alias, for example rs, py, go, js, ts")]
    language: Option<String>,

    #[arg(help = "Syntax concept to look up")]
    query: Vec<String>,

    #[arg(long, help = "Show topic metadata and related concepts")]
    more: bool,

    #[arg(long, help = "Print snippets only")]
    raw: bool,

    #[arg(long, help = "Print JSON")]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    Compare(OutputCommand),
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    Tui,
    Paths,
    Bench {
        #[arg(default_value = "hashmap iterate")]
        query: String,
    },
    Validate,
}

#[derive(Debug, Args)]
struct OutputCommand {
    #[arg(help = "Concept to compare across languages")]
    query: Vec<String>,

    #[arg(long, help = "Print snippets only")]
    raw: bool,

    #[arg(long, help = "Print JSON")]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum SessionCommand {
    Create {
        name: String,
        #[arg(short, long, default_value = "rust")]
        language: String,
    },
    List,
    Delete {
        name: String,
    },
    Switch {
        name: String,
    },
    Active,
    History,
    Bookmarks,
    Pins,
    ClearBookmarks,
    ClearPins,
    Stats,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if matches!(cli.command, Some(Command::Paths)) {
        println!("config: {}", default_config_dir()?.display());
        println!("data: {}", default_data_dir()?.display());
        return Ok(());
    }

    if matches!(cli.command, Some(Command::Validate)) {
        handle_validate()?;
        return Ok(());
    }

    if matches!(cli.command, Some(Command::Tui))
        || (cli.command.is_none() && cli.language.is_none())
    {
        let database = Database::open_default()?;
        let config = config::load()?;
        tui::run(&database, &config)?;
        return Ok(());
    }

    let database = Database::open_default()?;

    match cli.command {
        Some(Command::Compare(command)) => {
            let query = join_query(command.query)?;
            let results = compare::compare(&database, &query)?;
            if results.is_empty() {
                return Err(format!("no results for '{query}'").into());
            }
            println!("{}", compare::render(&results, command.raw, command.json)?);
        }
        Some(Command::Session { command }) => handle_session(&database, command)?,
        Some(Command::Paths) | Some(Command::Validate) => unreachable!(),
        Some(Command::Bench { query }) => handle_bench(&database, &query)?,
        Some(Command::Tui) => unreachable!(),
        None => handle_lookup(&database, cli)?,
    }

    Ok(())
}

fn handle_lookup(database: &Database, cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let language_input = cli.language.ok_or("missing language")?;
    let language = database
        .resolve_language(&search::normalize(&language_input))?
        .ok_or_else(|| {
            format!(
                "unknown language '{language_input}'. available: {}",
                database.list_languages().unwrap_or_default().join(", ")
            )
        })?;
    let query = join_query(cli.query)?;
    let results = search::search(database, Some(&language), &query)?;
    let language_results = results
        .into_iter()
        .filter(|result| result.language == language)
        .collect::<Vec<_>>();
    let result = best_result(&query, &language, &language_results)?;
    let active = session::ensure_default(database, &language)?;
    session::record_query(database, &active.name, &query)?;

    println!("{}", render_lookup(&result, cli.more, cli.raw, cli.json)?);
    Ok(())
}

fn handle_session(
    database: &Database,
    command: SessionCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        SessionCommand::Create { name, language } => {
            let normalized_language = database
                .resolve_language(&search::normalize(&language))?
                .ok_or_else(|| format!("unknown language '{language}'"))?;
            session::create(database, &name, &normalized_language)?;
            println!("created session {name} ({normalized_language})");
        }
        SessionCommand::List => {
            let sessions = session::list(database)?;
            if sessions.is_empty() {
                println!("no sessions");
            } else {
                for item in sessions {
                    println!("{}\t{}\t{}", item.name, item.language, item.created_at);
                }
            }
        }
        SessionCommand::Delete { name } => {
            session::delete(database, &name)?;
            println!("deleted session {name}");
        }
        SessionCommand::Switch { name } => {
            session::switch(database, &name)?;
            println!("active session {name}");
        }
        SessionCommand::Active => {
            if let Some(active) = session::active(database)? {
                println!("{}\t{}", active.name, active.language);
            } else {
                println!("no active session");
            }
        }
        SessionCommand::History => {
            let active = session::active(database)?.ok_or("no active session")?;
            for item in session::history(database, &active.name, 100)? {
                println!("{}\t{}", item.timestamp, item.query);
            }
        }
        SessionCommand::Bookmarks => {
            let active = session::active(database)?.ok_or("no active session")?;
            for item in session::saved_topics(database, "bookmarks", &active.name)? {
                println!("{item}");
            }
        }
        SessionCommand::Pins => {
            let active = session::active(database)?.ok_or("no active session")?;
            for item in session::saved_topics(database, "pins", &active.name)? {
                println!("{item}");
            }
        }
        SessionCommand::ClearBookmarks => {
            let active = session::active(database)?.ok_or("no active session")?;
            session::clear_saved_topics(database, "bookmarks", &active.name)?;
            println!("cleared bookmarks");
        }
        SessionCommand::ClearPins => {
            let active = session::active(database)?.ok_or("no active session")?;
            session::clear_saved_topics(database, "pins", &active.name)?;
            println!("cleared pins");
        }
        SessionCommand::Stats => {
            let active = session::active(database)?.ok_or("no active session")?;
            for (query, count) in session::stats(database, &active.name)? {
                println!("{count}\t{query}");
            }
        }
    }
    Ok(())
}

fn handle_bench(database: &Database, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cold_start = Instant::now();
    let fresh_database = Database::open_default()?;
    let cold_start_elapsed = cold_start.elapsed();

    let query_start = Instant::now();
    let results = search::search(database, Some("rust"), query)?;
    let query_elapsed = query_start.elapsed();

    let broad_start = Instant::now();
    let broad_results = search::search(&fresh_database, None, query)?;
    let broad_elapsed = broad_start.elapsed();

    println!("cold_start_ms={}", cold_start_elapsed.as_millis());
    println!("query_ms={}", query_elapsed.as_micros() as f64 / 1000.0);
    println!(
        "broad_query_ms={}",
        broad_elapsed.as_micros() as f64 / 1000.0
    );
    println!("results={}", results.len().max(broad_results.len()));
    Ok(())
}

fn handle_validate() -> Result<(), Box<dyn std::error::Error>> {
    let seed: SeedFile = serde_json::from_str(include_str!("../data/snippets.json"))?;
    let database = Database::open_memory_seeded()?;
    let mut snippets = 0usize;
    for language in &seed.languages {
        for topic in &language.topics {
            for snippet in &topic.snippets {
                snippets += 1;
                if snippet.lines().count() > 5 {
                    return Err(format!(
                        "snippet exceeds five lines: {} {}",
                        language.name, topic.title
                    )
                    .into());
                }
            }
        }
    }
    println!("languages={}", database.list_languages()?.len());
    println!("json_snippets={snippets}");
    println!("effective_snippets={}", database.count_snippets()?);
    println!("validation=ok");
    Ok(())
}

fn render_lookup(
    result: &SearchResult,
    more: bool,
    raw: bool,
    json: bool,
) -> Result<String, serde_json::Error> {
    if json {
        return serde_json::to_string_pretty(result);
    }
    if raw {
        return Ok(result.snippets.join("\n"));
    }
    let snippets = if std::io::stdout().is_terminal() {
        result
            .snippets
            .iter()
            .map(|snippet| highlight::highlight_cli(&result.language, snippet))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        result.snippets.join("\n")
    };
    if !more {
        return Ok(snippets);
    }

    let mut output = vec![
        format!("{} {}", "Topic:".dim(), result.topic.clone().bold()),
        format!("{} {}", "Language:".dim(), title_case(&result.language)),
        String::new(),
        snippets,
    ];
    if !result.related.is_empty() {
        output.push(String::new());
        output.push("Related:".dim().to_string());
        output.extend(result.related.iter().map(|topic| format!("- {topic}")));
    }
    Ok(output.join("\n"))
}

fn join_query(parts: Vec<String>) -> Result<String, Box<dyn std::error::Error>> {
    if parts.is_empty() {
        Err("missing query".into())
    } else {
        Ok(parts.join(" "))
    }
}

fn best_result(
    query: &str,
    language: &str,
    results: &[SearchResult],
) -> Result<SearchResult, Box<dyn std::error::Error>> {
    let Some(best) = results.first() else {
        return Err(format!("no results for '{query}' in {language}").into());
    };

    let tied = results
        .iter()
        .filter(|result| result.score == best.score)
        .take(4)
        .collect::<Vec<_>>();
    if tied.len() > 1 {
        let options = tied
            .iter()
            .map(|result| result.topic.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("ambiguous result for '{query}' in {language}: {options}").into());
    }

    Ok(best.clone())
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
