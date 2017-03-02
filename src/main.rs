#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

#[macro_use(bson, doc)]
extern crate bson;
extern crate jsonwebtoken as jwt;
extern crate rustc_serialize;
extern crate rustforum;
extern crate diesel;

use rocket_contrib::{JSON, Value};
use bson::Bson;
use std::sync::Arc;
use jwt::{encode, decode, Header, Algorithm};
use jwt::errors::{Error};
use rocket::Outcome;
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use self::rustforum::*;
use self::rustforum::models::*;
use self::diesel::prelude::*;
use std::time::SystemTime;
use diesel::insert;
use diesel::delete;
use diesel::update;

const SECRET_KEY: &'static str = "rahasia12345";

#[derive(Debug, RustcEncodable, RustcDecodable)]
struct Claims {
    sub: String,
    username: String,
    company: String
}

// Example validation implementation
impl Claims {
    fn is_valid(&self) -> bool {
        if self.company != "Codepolitan" {
            return false;
        }
        // expiration etc

        true
    }
}

struct Token (String);

impl<'a, 'r> FromRequest<'a, 'r> for Token {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Token, ()> {
        let keys: Vec<_> = request.headers().get("x-token").collect();
        
        if keys.len() != 1 {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        let key = keys[0];
        return Outcome::Success(Token(key.to_string()));
    }
}

#[derive(Deserialize)]
struct QuestionPayload {
    question_text: String,
    tags: String
}

#[derive(Serialize)]
struct QuestionRow {
    id: i32,
    question_text: String,
    tags: String,
    user_id: String
}

#[derive(Deserialize)]
struct SignupUserPayload {
    username: String,
    password: String,
    confirm_password: String,
    email: String,
}

#[derive(Deserialize)]
struct LoginUserPayload {
    username: String,
    password: String
}

#[derive(Serialize)]
struct UserRow {
    id: i32,
    username: String,
    email: String,
    token: String
}

/*

Question Service

*/

#[get("/")]
fn list_question(token: Token) -> JSON<Value> {

    let token_data = match decode::<Claims>(&token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };
    
    println!("{:?}", token_data.claims);
    println!("{:?}", token_data.header);
    println!("{:?}", token_data.claims.is_valid());

    use rustforum::schema::questions::dsl::*;

    let connection = establish_connection();

    let results = questions
        .load::<Question>(&connection)
        .expect("Error loading posts");

    println!("Found {} questions", results.len());
    
    let mut rows: Vec<QuestionRow> = vec![];

    for post in results {
        let question = QuestionRow {id: post.id, question_text: post.question_text, tags: post.tags, user_id: post.user_id};
        rows.push(question);
    }

    println!("Rows length: {}", rows.len());
    
    JSON(json!({
        "message": "Getting the question listss...",
        "data": rows
    }))
}

#[get("/<qid>")]
fn get_question(token: Token, qid: &str) -> JSON<Value> {

    let token_data = match decode::<Claims>(&token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };
    
    println!("{:?}", token_data.claims);
    println!("{:?}", token_data.header);
    println!("{:?}", token_data.claims.is_valid());

    use rustforum::schema::questions::dsl::*;

    let connection = establish_connection();

    println!("{}", qid);

    let row_id = qid.parse::<i32>().unwrap();

    let row = questions
        .find(row_id)
        .first::<Question>(&connection)
        .expect("Error loading posts");

    println!("{}", row.id);
    println!("{}", row.question_text);

	JSON(json!({
        "message": format!("Getting the question with id: {}", qid),
        "data": {
            "id": row.id,
            "question_text": row.question_text,
            "tags":row.tags,
            "user_id": row.user_id
        }
    }))

}

#[post("/", format = "application/json", data = "<question>")]
fn create_question(token: Token, question: JSON<QuestionPayload>) -> JSON<Value> {
    let quest: String = question.0.question_text;
    let tag: String = question.0.tags;
    let now = SystemTime::now();

    use rustforum::schema::questions::dsl::*;
    
    let connection = establish_connection();
    let mut uid = String::new();
    uid.push_str("12345");

    let new_question = NewQuestion { 
        question_text: quest, 
        tags: tag, 
        created_at: SystemTime::now(),
        user_id: uid 
    };

    insert(&new_question)
        .into(questions)
        .execute(&connection)
        .expect("Error saving new question");

    JSON(json!({
        "message": "Create the new question..",
        "data": {
            "question_text": format!("{}", new_question.question_text),
            "tags": format!("{}", new_question.tags)
        }
    }))
}

#[put("/<qid>", format = "application/json", data = "<question>")]
fn update_question(token: Token, qid: &str, question: JSON<QuestionPayload>) -> JSON<Value> {
    let quest = question.0.question_text;
    let tag = question.0.tags;

    use rustforum::schema::questions::dsl::*;

    let connection = establish_connection();

    let row_id = qid.parse::<i32>().unwrap();
    
    let row = update(
            questions
            .find(row_id)
        )
        .set(question_text.eq(quest))
        .get_result::<Question>(&connection)
        .expect("Error deleting question");

    JSON(json!({
        "message": "Create the new question..",
        "data": {
            "question_text": format!("{}", row.question_text),
            "tags": format!("{}", row.tags)
        }
    }))
}

#[delete("/<qid>")]
fn delete_question(qid: &str) -> JSON<Value> {

    use rustforum::schema::questions::dsl::*;

    let connection = establish_connection();

    let row_id = qid.parse::<i32>().unwrap();

    delete(
            questions
            .find(row_id)
        )
        .execute(&connection)
        .expect("Error deleting question");

    JSON(json!({
        "message": format!("Deleting the question with id: {}", qid)
    }))

}

#[post("/<qid>/answer")]
fn set_answer(qid: &str) -> JSON<Value> {
    JSON(json!({
        "message": "You call POST /question/<qid>/answer"
    }))
}

#[post("/<qid>/like")]
fn like_question(qid: &str) -> JSON<Value> {
    JSON(json!({
        "message": "You call POST /question/<qid>/like"
    }))
}

#[post("/<qid>/dislike")]
fn dislike_question(qid: &str) -> JSON<Value> {
    JSON(json!({
        "message": "You call POST /question/<qid>/dislike"
    }))
}


/*

Answer Service

*/

#[get("/<qid>")]
fn get_answer(qid: &str) -> JSON<Value> {
    JSON(json!({
        "message": "You call GET /answer/<qid>"
    }))
}

#[put("/<qid>")]
fn update_answer(qid: &str) -> JSON<Value> {
    JSON(json!({
        "message": "You call PUT /answer/<qid>"
    }))
}

#[delete("/<qid>")]
fn delete_answer(qid: &str) -> JSON<Value> {
    JSON(json!({
        "message": "You call DELETE /answer/<qid>"
    }))
}

#[post("/<qid>/like")]
fn like_answer(qid: &str) -> JSON<Value> {
    JSON(json!({
        "message": "You call POST /answer/<qid>/like"
    }))
}

#[post("/<qid>/dislike")]
fn dislike_answer(qid: &str) -> JSON<Value> {
    JSON(json!({
        "message": "You call POST /answer/<qid>/dislike"
    }))
}


/*

Main Service

*/

#[post("/login", format = "application/json", data = "<user>")]
fn login(user: JSON<LoginUserPayload>) -> JSON<Value> {
    let v_username: String = user.0.username;
    let v_password: String = user.0.password;

    // get user
    // if has token and not expired use that token
    // if token has expired generate a new one

    let my_claims = Claims {
        username: v_username.to_owned(),
        sub: "ridwanbejo@gmail.com".to_owned(),
        company: "Codepolitan".to_owned()
    };

    let v_token = match encode(Header::default(), &my_claims, SECRET_KEY.as_ref()) {
        Ok(t) => t,
        Err(_) => panic!() // in practice you would return the error
    };

    JSON(json!({
        "message": "Login success",
        "data": {
            "token": v_token
        }
    }))
}

#[post("/signup", format = "application/json", data = "<user>")]
fn signup(user: JSON<SignupUserPayload>) -> JSON<Value> {

    let v_username: String = user.0.username;
    let v_password: String = user.0.password;
    let v_email: String = user.0.email;

    // check if user with the given username and/or email has existed
    // if unexist create a new one
        // if password with confirm_password is unmatch then error
        // else Signup success
    // else Signup error

    let my_claims = Claims {
        username: v_username.to_owned(),
        sub: v_email.to_owned(),
        company: "Codepolitan".to_owned()
    };

    let v_token = match encode(Header::default(), &my_claims, SECRET_KEY.as_ref()) {
        Ok(t) => t,
        Err(_) => panic!() // in practice you would return the error
    };

    println!("{:?}", v_token);

    use rustforum::schema::users::dsl::*;
    
    let connection = establish_connection();

    let new_user = NewUser { 
        username: v_username, 
        email: v_email, 
        password: v_password, 
        token: v_token,
        created_at: SystemTime::now(),
    };

    insert(&new_user)
        .into(users)
        .execute(&connection)
        .expect("Error saving new user");


    JSON(json!({
        "message": "Signup success",
        "data": {
            "token":new_user.token
        }
    }))
}

#[get("/logout")]
fn logout() -> JSON<Value> {
    JSON(json!({
        "message": "You call /logout"
    }))
}

#[get("/change_password")]
fn change_password() -> JSON<Value> {
    JSON(json!({
        "message": "You call /change_password"
    }))
}

#[get("/forgot_password")]
fn forgot_password() -> JSON<Value> {
    JSON(json!({
        "message": "You call /forgot_password"
    }))
}

#[get("/change_profile_picture")]
fn change_profile_picture() -> JSON<Value> {
    JSON(json!({
        "message": "You call /change_profile_picture"
    }))
}

#[get("/")]
fn index() -> JSON<Value> {

    JSON(json!({
        "message": "Welcome to RustForum API :D"
    }))
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, login, logout, signup, change_password, forgot_password, change_profile_picture])
        .mount("/question", routes![list_question, get_question, create_question, update_question, delete_question, like_question, dislike_question, set_answer ])
    	.mount("/answer", routes![get_answer, update_answer,  delete_answer, like_answer, dislike_answer ])
    	.launch();
}
