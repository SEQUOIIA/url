use clap::{arg, command, Command};
use crate::commands::CommandData;
use crate::commands::delete::Delete;
use crate::commands::key::Key;
use crate::commands::new::New;

mod config;
mod model;
mod commands;

fn main() {
    let mut app = command!()
        .author("Emil H. Clausen - github.com/SEQUOIIA")
        .arg(arg!(
            -d --debug ... "Turn debugging information on"
        ))
        .subcommand(
            Command::new("new")
                .about("Create new short URL")
                .arg_required_else_help(true)
                .arg(arg!(-n --"name" <NAME> "Optional custom name").required(false))
                .arg(arg!(-a --"api-key" <APIKEY> "Optionally specify API key. Can also be set via environment variable (SEQ_URL_API_KEY) or config file").required(false))
                .arg(arg!([URL]))

        )
        .subcommand(
            Command::new("delete")
                .about("Delete short URL")
                .arg_required_else_help(true)
                .arg(arg!(-a --"api-key" <APIKEY> "Optionally specify API key. Can also be set via environment variable (SEQ_URL_API_KEY) or config file").required(false))
                .arg(arg!([NAME]))

        )
        .subcommand(
            Command::new("key")
                .about("Manage API keys")
                .arg_required_else_help(true)
                .subcommands( vec![
                    Command::new("create")
                        .about("Create a new API key")
                        .arg(arg!(-d --description <DESCRIPTION> "Sets description of the created API key. Example: 'This key belongs to x user'").required(false)),
                    Command::new("delete")
                        .about("Delete existing API key")
                        .arg(arg!([Key])),
                    Command::new("list")
                        .about("List API keys")
                ])
        );

    let matches = app.clone().get_matches();
    let conf = config::Config::load().unwrap();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let context = CommandData::new(app.clone(), sub_matches.clone(), conf);
            New::handle(context);
        },
        Some(("delete", sub_matches)) => {
            let context = CommandData::new(app.clone(), sub_matches.clone(), conf);
            Delete::handle(context);
        },
        Some(("key", sub_matches)) => {
            let context = CommandData::new(app.clone(), sub_matches.clone(), conf);
            Key::handle(context);
        },
        _ => {
            app.print_help();
        }
    }
}
