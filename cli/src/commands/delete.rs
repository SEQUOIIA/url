use crate::commands::{CommandData, new_runtime};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use reqwest::{Error, Response};

pub struct Delete;

impl<'a> Delete {
    pub fn handle(context : CommandData) {
        // Command logic
        new_runtime().block_on(async move {
            let client = reqwest::Client::new();

            let req_data = RequestData {
                id: context.arg_matches.value_of("NAME").unwrap().to_owned(),
            };

            let resp = match client.delete(format!("{}/delete", context.conf.api_endpoint))
                .header("x-api-key", context.conf.get_api_key().unwrap())
                .json(&req_data)
                .send()
                .await {
                Ok(val) => val,
                Err(err) => {
                    println!("{}", err);
                    return ();
                }
            };

            if resp.status().eq(&401) {
                println!("{}", "Unauthorised");
                return ();
            }

            println!("{}", resp.text().await.unwrap());
            ()
        });

    }
}

#[derive(Serialize, Deserialize)]
struct RequestData {
    id : String
}