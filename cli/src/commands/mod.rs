use clap::{App, ArgMatches};
use tokio::runtime::Runtime;

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

pub fn new_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("seqtf_url-worker")
        .enable_all()
        .build().unwrap()
}