use clap::{App, ArgMatches};

pub mod new;

pub struct CommandData<'a> {
    app: App<'a>,
    arg_matches : ArgMatches
}

impl<'a, 'b> CommandData<'a> {
    pub fn new(app : App<'a>, arg_matches : ArgMatches) -> Self {
        Self {
            app,
            arg_matches,
        }
    }
}