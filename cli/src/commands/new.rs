use crate::commands::{CommandData, new_runtime};
use serde::{Serialize, Deserialize};

pub struct New;

impl<'a> New {
    pub fn handle(context : CommandData) {
        // Command logic
        new_runtime().block_on(async move {
            let client = reqwest::Client::new();

            let req_data = RequestData {
                url: context.arg_matches.value_of("URL").unwrap().to_owned()
            };

            let resp = client.post("http://localhost:8380/new")
                .header("x-api-key", "")
                .json(&req_data)
                .send()
                .await.unwrap();

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
    url : String
}