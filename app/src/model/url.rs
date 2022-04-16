use crate::schema::{urls};
use serde::{Serialize, Deserialize};

pub struct Url {
    pub id : String,
    pub url : String
}

#[derive(Serialize, Deserialize)]
pub struct UrlRequest {
    pub id : Option<String>,
    pub url : String,
}

impl From<UrlDb> for Url {
    fn from(u: UrlDb) -> Self {
        Self {
            id: u.id,
            url: u.url
        }
    }
}
impl PartialEq for Url {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.url == other.url
    }
}

#[derive(Queryable, Identifiable)]
#[table_name="urls"]
pub struct UrlDb {
    pub id : String,
    pub url : String
}

impl From<Url> for UrlDb {
    fn from(u: Url) -> Self {
        Self {
            id: u.id,
            url: u.url
        }
    }
}

#[derive(Insertable)]
#[table_name="urls"]
pub struct UrlDbInsert {
    pub id : String,
    pub url : String
}
