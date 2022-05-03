use crate::commands::{CommandData, new_runtime};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use reqwest::{Error, Response};

pub struct Key;

impl<'a> Key {
    pub fn handle(context : CommandData) {
        // Command logic
        match context.arg_matches.subcommand() {
            Some(("create", sub_matches)) => {

            },
            Some(("delete", sub_matches)) => {

            },
            _ => {}
        }
    }
}

