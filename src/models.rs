use crate::schema::posts;
use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct Users {
    pub id: i32,
    pub username: String,
    pub password: String,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostPayload {
    pub title: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPayload {
    pub username: String,
    pub password: String,
}

//#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
//struct CreateUserBody {
//    username: String,
//    password: String,
//}
//
//#[derive(Queryable, Selectable, Serialize, Deserialize, Debug)]
//struct UserNoPassword {
//    id: i32,
//    username: String,
//}
