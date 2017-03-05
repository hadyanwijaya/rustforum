Rust Forum
==========

Actually i'm a programmer who like to code with Python and PHP but i want to try the excelently of Rust Lang. This is the sample project for Rust Lang using Rocket Framework. This repository is contain RESTful API for forum engine. These feature is including:

* Question
* Answer
* Like / Dislike
* User Registration
* Change Password
* Forgot Password
* Change Profile Picture
* Authentication and Authorization using Json Web TOken

Useful links:

* http://mongoc.org/libmongoc/current/installing.html
* https://github.com/thijsc/mongo-rust-driver
* http://thijsc.github.io/mongo-rust-driver/mongo_driver/


```
$ cargo run

Finished dev [unoptimized + debuginfo] target(s) in 11.6 secs
    Running `target/debug/rustforum`
🔧  Configured for development.
    => address: localhost
    => port: 8000
    => log: normal
    => workers: 4
🛰  Mounting '/':
    => GET /
    => POST /login application/json
    => GET /logout
    => POST /signup application/json
    => GET /change_password
    => GET /forgot_password
    => GET /change_profile_picture
🛰  Mounting '/question':
    => GET /question/<qid>/answer
    => GET /question/my
    => GET /question
    => GET /question/<qid>
    => POST /question application/json
    => PUT /question/<qid> application/json
    => DELETE /question/<qid>
    => POST /question/<qid>/like
    => POST /question/<qid>/dislike
    => POST /question/<qid>/answer
🛰  Mounting '/answer':
    => PUT /answer/<qid>
    => DELETE /answer/<qid>
    => POST /answer/<qid>/like
    => POST /answer/<qid>/dislike
🚀  Rocket has launched from http://localhost:8000...
```