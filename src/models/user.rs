use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct User {
//     #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
//     pub id: Option<ObjectId>,
//     pub username: String,
//     pub email: String,
//     pub password_hash: String,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    pub password: String,
    pub refresh_token: Option<String>,
}