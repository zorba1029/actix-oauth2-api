use mongodb::{Client, Database};
use std::env;

pub async fn connect_db() -> Database {
    let mogodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client = Client::with_uri_str(mogodb_uri)
        .await
        .expect("Failed to connect to MongoDB");

    client.database("rust_oauth2")
}
