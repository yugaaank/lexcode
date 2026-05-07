mod cli;
mod compare;
mod config;
mod db;
mod logging;
mod models;
mod search;
mod session;
mod tui;

fn main() {
    logging::debug("codelex start");
    if let Err(error) = cli::run() {
        logging::error(&error.to_string());
        eprintln!("codelex: {error}");
        std::process::exit(1);
    }
}
