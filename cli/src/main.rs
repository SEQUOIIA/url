use clap::{arg, command, Command};
use crate::commands::CommandData;
use crate::commands::new::New;

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
                .arg(arg!([URL]))
                .arg(arg!(-n --name "Optional custom name"))
        );

    let matches = app.clone().get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let context = CommandData::new(app.clone(), sub_matches.clone());
            New::handle(context);
        },
        _ => {
            app.print_help();
        }
    }
}
