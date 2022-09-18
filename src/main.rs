use clap::{arg, App, Command};
use comfy_table::Table;
use exitcode;
use model::Collection;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod model;

fn main() {
    let version: &str = env!("CARGO_PKG_VERSION");

    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut collection = Collection::new();
    debug!("collection: {:?}", collection);

    let args = App::new("lin-help")
        .version(version)
        .about("a handy tool for collecting common shell commands")
        .subcommand(
            Command::new("add")
                .about("adds a new command to the list")
                .args(&[
                    arg!(<COMMAND> "command to save, should be in quotes"),
                    arg!(<DESCRIPTION> "short description of the provided command"),
                ]),
        )
        .subcommand(
            Command::new("search")
                .about("shows all available commands for the search term")
                .arg(arg!(<TERM> "command or description to search for")),
        )
        .subcommand(Command::new("list").about("list all available commands"))
        .get_matches();

    match args.subcommand() {
        Some(("add", args)) => {
            let command = args.value_of("COMMAND").unwrap();
            let description = args.value_of("DESCRIPTION").unwrap();
            let entry = model::Entry::new(command.to_string(), description.to_string());

            collection.save(entry);
        }
        Some(("search", args)) => {
            let term = args.value_of("TERM").unwrap();

            // TODO dedup table creation
            let mut table = Table::new();
            table.set_header(vec!["ID", "Command", "Description"]);

            for entry in collection.get_entries() {
                if entry.command.contains(term) || entry.description.contains(term) {
                    table.add_row(vec![
                        entry.id.to_string(),
                        entry.command.clone(),
                        entry.description.clone(),
                    ]);
                }
            }
            info!("\n{}", table);
        }
        Some(("list", _)) => {
            let mut table = Table::new();
            table.set_header(vec!["ID", "Command", "Description"]);

            for entry in collection.get_entries() {
                table.add_row(vec![
                    entry.id.to_string(),
                    entry.command.to_string(),
                    entry.description.to_string(),
                ]);
            }
            info!("\n{}", table);
        }
        _ => {
            error!("unable to recognize subcommand");
            std::process::exit(exitcode::USAGE);
        }
    }
}
