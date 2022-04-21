table! {
    api_keys (id) {
        id -> Integer,
        key -> Text,
        description -> Nullable<Text>,
    }
}

table! {
    urls (id) {
        id -> Text,
        url -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    api_keys,
    urls,
);
