use crate::commands::{CommandData, new_runtime};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use reqwest::{Error, Response};

pub struct Key;

impl<'a> Key {
    pub fn handle(context : CommandData) {
        // Command logic
        match context.arg_matches.subcommand() {
            Some(("list", sub_matches)) => {
                new_runtime().block_on(async move {
                    let client = reqwest::Client::new();
                    let resp = match client.get(format!("{}/key", context.conf.api_endpoint))
                        .header("x-api-key", context.conf.get_api_key().unwrap())
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

                    let resp_bytes = resp.bytes().await.unwrap();
                    let keys : Vec<ApiKey> = serde_json::from_slice(resp_bytes.as_ref()).unwrap();

                    for key in keys {
                        println!("Key: '{}' - \"{}\"", key.key, key.description.unwrap_or("".to_owned()))
                    }

                    ()
                });
            },
            Some(("create", sub_matches)) => {
                new_runtime().block_on(async move {
                    let client = reqwest::Client::new();

                    let req_data = CreateRequestData {
                        description: sub_matches.value_of("description").unwrap_or("").to_owned()
                    };

                    let resp = match client.post(format!("{}/key", context.conf.api_endpoint))
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
            },
            Some(("delete", sub_matches)) => {
                new_runtime().block_on(async move {
                    let client = reqwest::Client::new();

                    let req_data = DeleteRequestData {
                        key: sub_matches.value_of("Key").unwrap_or("").to_owned()
                    };

                    let resp = match client.delete(format!("{}/key", context.conf.api_endpoint))
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

                    match resp.status().as_u16() {
                        401 => {
                            println!("{}", "Unauthorised");
                            return ();
                        }
                        200 => {
                            println!("Key deleted");
                            return ();
                        },
                        404 => {
                            println!("Key not found");
                            return ();
                        }
                        _ => {

                        }
                    }
                    ()
                });
            },
            _ => {}
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CreateRequestData {
    description : String
}

#[derive(Serialize, Deserialize)]
struct DeleteRequestData {
    key : String
}

#[derive(Serialize, Deserialize)]
pub struct ApiKey {
    pub id : i64,
    pub key : String,
    pub description : Option<String>
}
