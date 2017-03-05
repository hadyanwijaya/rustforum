use std::time::SystemTime;


#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub token: String,
    pub created_at: SystemTime,
}

use super::schema::users;

#[derive(Debug, Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub token: String,
    pub created_at: SystemTime,
}

#[derive(Debug, Queryable)]
pub struct Question {
    pub id: i32,
    pub question_text: String,
    pub tags: String,
    pub created_at: SystemTime,
    pub user_id: i32
}

use super::schema::questions;

#[derive(Debug, Insertable)]
#[table_name="questions"]
pub struct NewQuestion {
    pub question_text: String,
    pub tags: String,
    pub created_at: SystemTime,
    pub user_id: i32
}

// #[derive(Debug, Queryable)]
// pub struct Answer {
//     pub id: i32,
//     pub answer_text: String,
//     pub question_id: String,
//     pub user_id: String,
//     pub created_at: SystemTime
// }

// use super::schema::questions;

// #[derive(Debug, Insertable)]
// #[table_name="answers"]
// pub struct NewAnswer {
//     pub answer_text: String,
//     pub question_id: String,
//     pub user_id: String,
//     pub created_at: SystemTime
// }

// #[derive(Debug, Queryable)]
// pub struct Reaction {
//     pub id: i32,
//     pub answer_text: String,
//     pub question_id: String,
//     pub user_id: String,
//     pub created_at: SystemTime
// }

// use super::schema::questions;

// #[derive(Debug, Insertable)]
// #[table_name="answers"]
// pub struct NewAnswer {
//     pub answer_text: String,
//     pub question_id: String,
//     pub user_id: String,
//     pub created_at: SystemTime
// }

