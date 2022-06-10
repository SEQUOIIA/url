use crate::commands::{CommandData, new_runtime};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use reqwest::{Error, Response};

pub struct New;

impl<'a> New {
    pub fn handle(context : CommandData) {
        // Command logic
        new_runtime().block_on(async move {
            let client = reqwest::Client::new();

            let req_data = RequestData {
                url: context.arg_matches.value_of("URL").unwrap().to_owned(),
                id: context.arg_matches.value_of("name").map_or(None, |val| {
                    Some(val.to_owned())
                })
            };

            let resp = match client.post(format!("{}/new", context.conf.api_endpoint))
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
    url : String,
    id : Option<String>
}