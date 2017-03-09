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
    question_id: i32,
    question_text: String,
    tags: String,
    user_id: i32
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

#[derive(Deserialize)]
struct ChangePasswordUserPayload {
    password: String,
    new_password: String,
    confirm_new_password: String,
}

#[derive(Serialize)]
struct UserRow {
    id: i32,
    username: String,
    email: String,
    token: String
}

#[derive(Deserialize)]
struct AnswerPayload {
    answer_text: String,
}

#[derive(Serialize)]
struct AnswerRow {
    answer_id: i32, 
    answer_text: String,
    question_id: i32,
    user_id: i32,
}


#[derive(Serialize)]
struct ReactionRow {
    reaction_id: i32, 
    user_id: i32,
    object_id: i32,
    object_type: String,
    reaction_type: String
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
    
    if (token_data.claims.is_valid())
    {

        // get question
        use rustforum::schema::questions::dsl::*;
        let connection = establish_connection();

        let results = questions
            .load::<Question>(&connection)
            .expect("Error loading posts");

        let mut rows: Vec<QuestionRow> = vec![];

        for post in results {
            let question = QuestionRow {question_id: post.question_id, question_text: post.question_text, tags: post.tags, user_id: post.user_id};
            rows.push(question);
        }

        println!("Rows length: {}", rows.len());
        
        JSON(json!({
            "message": "Getting the question listss...",
            "data": rows
        }))

    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[get("/my")]
fn my_question(v_token: Token) -> JSON<Value> {

    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };
    
    println!("{}", token_data.claims.username);
    println!("{}", token_data.claims.sub);
    println!("{}", token_data.claims.is_valid());
    
    if (token_data.claims.is_valid())
    {
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");
                    
        // get question
        use rustforum::schema::questions::dsl::*;
        let connection = establish_connection();

        let results = questions
            .filter(user_id.eq(v_user.id))
            .load::<Question>(&connection)
            .expect("Error loading posts");

        let mut rows: Vec<QuestionRow> = vec![];

        for post in results {
            let question = QuestionRow {question_id: post.question_id, question_text: post.question_text, tags: post.tags, user_id: post.user_id};
            rows.push(question);
        }

        println!("Rows length: {}", rows.len());
        
        JSON(json!({
            "message": "Getting the question listss...",
            "data": rows
        }))

    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
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
    
    if (token_data.claims.is_valid())
    {
        use rustforum::schema::questions::dsl::*;

        let connection = establish_connection();
        let row_id = qid.parse::<i32>().unwrap();
        let row = questions
            .find(row_id)
            .first::<Question>(&connection)
            .expect("Error loading posts");

        JSON(json!({
            "message": format!("Getting the question with id: {}", qid),
            "data": {
                "id": row.question_id,
                "question_text": row.question_text,
                "tags":row.tags,
                "user_id": row.user_id
            }
        }))
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[post("/", format = "application/json", data = "<question>")]
fn create_question(v_token: Token, question: JSON<QuestionPayload>) -> JSON<Value> {

    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };
    
    if (token_data.claims.is_valid())
    {
        let quest: String = question.0.question_text;
        let tag: String = question.0.tags;
        let now = SystemTime::now();

        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        use rustforum::schema::questions::dsl::*;
        let connection = establish_connection();
        
        let new_question = NewQuestion { 
            question_text: quest, 
            tags: tag, 
            created_at: SystemTime::now(),
            user_id: v_user.id 
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
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[put("/<qid>", format = "application/json", data = "<question>")]
fn update_question(v_token: Token, qid: &str, question: JSON<QuestionPayload>) -> JSON<Value> {

    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        let quest = question.0.question_text;
        let tag = question.0.tags;
        let row_id = qid.parse::<i32>().unwrap();
        
        use rustforum::schema::questions::dsl::*;
        let connection = establish_connection();
        
        let v_question = questions
                .filter(user_id.eq(v_user.id))
                .filter(question_id.eq(row_id))
                .load::<Question>(&connection)
                .expect("Error loading users");

        if (v_question.len() > 0){
            let row = update(
                    questions
                    .find(row_id)
                )
                .set(question_text.eq(quest))
                .get_result::<Question>(&connection)
                .expect("Error updating question");

            JSON(json!({
                "message": "Update question is success..",
                "data": {
                    "question_text": format!("{}", row.question_text),
                    "tags": format!("{}", row.tags)
                }
            }))
        }
        else
        {
            JSON(json!({
                "message": "You are unauthorized to edit this question"
            })) 
        }
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[delete("/<qid>")]
fn delete_question(v_token: Token, qid: &str) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        let row_id = qid.parse::<i32>().unwrap();
        
        use rustforum::schema::questions::dsl::*;
        let connection = establish_connection();
        
        let v_question = questions
                .filter(user_id.eq(v_user.id))
                .filter(question_id.eq(row_id))
                .load::<Question>(&connection)
                .expect("Error loading question");

        println!("{}", v_question.len());

        if (v_question.len() > 0){
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
        else
        {
            JSON(json!({
                "message": "You are unauthorized to delete this question"
            })) 
        }
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[get("/<qid>/answer")]
fn get_answer(v_token: Token, qid: &str) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        let v_question_id = qid.parse::<i32>().unwrap();
        
        // get question
        use rustforum::schema::answers::dsl::*;
        let connection = establish_connection();

        let results = answers
            .filter(question_id.eq(v_question_id))
            .load::<Answer>(&connection)
            .expect("Error loading answer");

        let mut rows: Vec<AnswerRow> = vec![];

        for post in results {
            let answ = AnswerRow {answer_id: post.answer_id, answer_text: post.answer_text, question_id: post.question_id, user_id: post.user_id};
            rows.push(answ);
        }

        println!("Rows length: {}", rows.len());
        
        JSON(json!({
            "message": "Getting the answer lists...",
            "data": rows
        }))
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[post("/<qid>/answer", format = "application/json", data = "<answer>")]
fn set_answer(v_token: Token, qid: &str, answer: JSON<AnswerPayload>) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        let v_answer: String = answer.0.answer_text;
        let v_question_id = qid.parse::<i32>().unwrap();
        
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        use rustforum::schema::answers::dsl::*;
        let connection = establish_connection();
        
        let new_answer = NewAnswer { 
            answer_text: v_answer, 
            question_id: v_question_id, 
            user_id: v_user.id ,
            created_at: SystemTime::now(),
        };

        insert(&new_answer)
            .into(answers)
            .execute(&connection)
            .expect("Error saving new answer");

        JSON(json!({
            "message": "Create the new answer is success..",
            "data": {
                "answer_text": format!("{}", new_answer.answer_text),
                "question_id": format!("{}", new_answer.question_id)
            }
        }))
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[post("/<qid>/like")]
fn like_question(v_token: Token, qid: &str) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        let v_object_type = "question".to_string();
        let v_reaction_type = "like".to_string();
        let v_object_id = qid.parse::<i32>().unwrap();
        
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        use rustforum::schema::reactions::dsl::*;
        let connection = establish_connection();
        
        let new_reaction = NewReaction { 
            object_type: v_object_type, 
            object_id: v_object_id, 
            reaction_type: v_reaction_type,
            user_id: v_user.id,
            created_at: SystemTime::now(),
        };

        insert(&new_reaction)
            .into(reactions)
            .execute(&connection)
            .expect("Error saving reaction");

        JSON(json!({
            "message": "Give like reaction is success..",
            "data": {
                "object_type": format!("{}", new_reaction.object_type),
                "object_id": format!("{}", new_reaction.object_id),
                "reaction_type": format!("{}", new_reaction.reaction_type)
            }
        }))
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[post("/<qid>/dislike")]
fn dislike_question(v_token: Token, qid: &str) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        let v_object_type = "question".to_string();
        let v_reaction_type = "dislike".to_string();
        let v_object_id = qid.parse::<i32>().unwrap();
        
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        use rustforum::schema::reactions::dsl::*;
        let connection = establish_connection();
        
        let new_reaction = NewReaction { 
            object_type: v_object_type, 
            object_id: v_object_id, 
            reaction_type: v_reaction_type,
            user_id: v_user.id,
            created_at: SystemTime::now(),
        };

        insert(&new_reaction)
            .into(reactions)
            .execute(&connection)
            .expect("Error saving reaction");

        JSON(json!({
            "message": "Give dislike reaction is success..",
            "data": {
                "object_type": format!("{}", new_reaction.object_type),
                "object_id": format!("{}", new_reaction.object_id),
                "reaction_type": format!("{}", new_reaction.reaction_type)
            }
        }))
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}


/*

Answer Service

*/

/* TODO */
#[put("/<qid>", format = "application/json", data = "<answer>")]
fn update_answer(v_token: Token, qid: &str, answer: JSON<AnswerPayload>) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        let answ = answer.0.answer_text;
        let row_id = qid.parse::<i32>().unwrap();
        
        use rustforum::schema::answers::dsl::*;
        let connection = establish_connection();
        
        let v_answer = answers
                .filter(user_id.eq(v_user.id))
                .filter(answer_id.eq(row_id))
                .load::<Answer>(&connection)
                .expect("Error loading users");

        if (v_answer.len() > 0){
            let row = update(
                    answers
                    .find(row_id)
                )
                .set(answer_text.eq(answ))
                .get_result::<Answer>(&connection)
                .expect("Error updating answer");

            JSON(json!({
                "message": "Update answer is success..",
                "data": {
                    "answer_text": format!("{}", row.answer_text),
                }
            }))
        }
        else
        {
            JSON(json!({
                "message": "You are unauthorized to edit this answer"
            })) 
        }
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[delete("/<qid>")]
fn delete_answer(v_token: Token, qid: &str) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        let row_id = qid.parse::<i32>().unwrap();
        
        use rustforum::schema::answers::dsl::*;
        let connection = establish_connection();
        
        let v_answer = answers
                .filter(user_id.eq(v_user.id))
                .filter(answer_id.eq(row_id))
                .load::<Answer>(&connection)
                .expect("Error loading question");

        println!("{}", v_answer.len());

        if (v_answer.len() > 0){
            delete(
                    answers
                    .find(row_id)
                )
                .execute(&connection)
                .expect("Error deleting answer");

            JSON(json!({
                "message": format!("Deleting the answer with id: {}", qid)
            }))

        }
        else
        {
            JSON(json!({
                "message": "You are unauthorized to delete this answer"
            })) 
        }
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

/* TODO */
#[post("/<qid>/like")]
fn like_answer(v_token: Token, qid: &str) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    use rustforum::schema::users::dsl::*;
    let connection = establish_connection();

    // get user id
    let v_user = users
                .filter(username.eq(token_data.claims.username))
                .filter(email.eq(token_data.claims.sub))
                .first::<User>(&connection)
                .expect("Error loading users");

    JSON(json!({
        "message": "You call POST /answer/<qid>/like"
    }))
}

/* TODO */
#[post("/<qid>/dislike")]
fn dislike_answer(v_token: Token, qid: &str) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    use rustforum::schema::users::dsl::*;
    let connection = establish_connection();

    // get user id
    let v_user = users
                .filter(username.eq(token_data.claims.username))
                .filter(email.eq(token_data.claims.sub))
                .first::<User>(&connection)
                .expect("Error loading users");

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
    use rustforum::schema::users::dsl::*;
            
    let connection = establish_connection();
             
    let results = users
                .filter(username.eq(v_username.clone()))
                .filter(password.eq(v_password.clone()))
                .load::<User>(&connection)
                .expect("Error loading users");

    // println!("Found {} users", results.len());

    if (results.len() > 0) {

        let v_user = users
                .filter(username.eq(v_username.clone()))
                .filter(password.eq(v_password.clone()))
                .first::<User>(&connection)
                .expect("Error loading users");

        let my_claims = Claims {
            username: v_user.username.to_owned(),
            sub: v_user.email.to_owned(),
            company: "Codepolitan".to_owned()
        };

        let v_token = match encode(Header::default(), &my_claims, SECRET_KEY.as_ref()) {
            Ok(t) => t,
            Err(_) => panic!() // in practice you would return the error
        };

        let row = update(
                users
                .find(v_user.id)
            )
            .set(token.eq(v_token.clone()))
            .get_result::<User>(&connection)
            .expect("Error updating users");


        JSON(json!({
            "message": "Login success",
            "data": {
                "token": v_token
            }
        }))
    }

    else
    {
        JSON(json!({
            "message": "Login failed. User not found.."
        }))
    }
}

#[post("/signup", format = "application/json", data = "<user>")]
fn signup(user: JSON<SignupUserPayload>) -> JSON<Value> {

    let v_username: String = user.0.username;
    let v_password: String = user.0.password;
    let v_confirm_password: String = user.0.confirm_password;
    let v_email: String = user.0.email;


    // check if user with the given username and/or email has existed
    // if unexist create a new one
    
    use rustforum::schema::users::dsl::*;
            
    let connection = establish_connection();
             
    let results = users
                .filter(email.eq(v_email.clone()))
                .filter(username.eq(v_username.clone()))
                .load::<User>(&connection)
                .expect("Error loading users");

    if (results.len() == 0) {
        // if password with confirm_password is unmatch then error
        if (v_password != v_confirm_password)
        {
            JSON(json!({
                "message": "Signup failed. Password and Confirm Password is not match.."
            }))
        }
        else
        {
            // else Signup success
        
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
    }
    else {
        // else Signup error
        JSON(json!({
            "message": "Signup failed. Account is unavailable.."
        }))
    }
}

#[post("/change_password", format = "application/json", data = "<user>")]
fn change_password(v_token: Token, user: JSON<ChangePasswordUserPayload>) -> JSON<Value> {

    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        let v_password: String = user.0.password;
        let v_new_password: String = user.0.new_password;
        let v_confirm_new_password: String = user.0.confirm_new_password;

        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");
        if (v_user.password == v_password.clone())
        {
            if (v_new_password == v_confirm_new_password)
            {
                let row = update(
                    users
                    .find(v_user.id)
                )
                .set(password.eq(v_new_password))
                .get_result::<User>(&connection)
                .expect("Error updating users");

                JSON(json!({
                    "message": "Change password success..",
                }))  
            }
            else
            {
                JSON(json!({
                    "message": "New password with confirm new password is unmatch",
                }))
            }
        }
        else
        {
            JSON(json!({
                "message": "Invalid account",
            }))
        }
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

#[get("/logout")]
fn logout(v_token: Token) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    if (token_data.claims.is_valid())
    {
        use rustforum::schema::users::dsl::*;
        let connection = establish_connection();

        // get user id
        let v_user = users
                    .filter(username.eq(token_data.claims.username))
                    .filter(email.eq(token_data.claims.sub))
                    .first::<User>(&connection)
                    .expect("Error loading users");

        let row = update(
                users
                .find(v_user.id)
            )
            .set(token.eq(""))
            .get_result::<User>(&connection)
            .expect("Error updating users");

        JSON(json!({
            "message": "Logout success..",
        }))
    }
    else
    {
        JSON(json!({
            "message": "Invalid token"
        }))
    }
}

/* TODO */
#[get("/forgot_password")]
fn forgot_password(v_token: Token) -> JSON<Value> {
    let token_data = match decode::<Claims>(&v_token.0, SECRET_KEY.as_ref(), Algorithm::HS256) {
        Ok(c) => c,
        Err(err) => match err {
            Error::InvalidToken => panic!(),
            _ => panic!()
        }
    };

    use rustforum::schema::users::dsl::*;
    let connection = establish_connection();

    // get user id
    let v_user = users
                .filter(username.eq(token_data.claims.username))
                .filter(email.eq(token_data.claims.sub))
                .first::<User>(&connection)
                .expect("Error loading users");

    /* TODO */

    JSON(json!({
        "message": "You call /forgot_password"
    }))
}

/* TODO */
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
        .mount("/question", routes![get_answer, my_question, list_question, get_question, create_question, update_question, delete_question, like_question, dislike_question, set_answer ])
    	.mount("/answer", routes![update_answer,  delete_answer, like_answer, dislike_answer ])
    	.launch();
}
