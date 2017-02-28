use std::time::SystemTime;

#[derive(Debug, Queryable)]
pub struct Question {
    pub id: i32,
    pub question_text: String,
    pub tags: String,
    pub created_at: SystemTime,
    pub user_id: String
}

use super::schema::questions;

#[derive(Debug, Insertable)]
#[table_name="questions"]
pub struct NewQuestion {
    pub question_text: String,
    pub tags: String,
    pub created_at: SystemTime,
    pub user_id: String
}