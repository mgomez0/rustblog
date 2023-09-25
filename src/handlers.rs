use super::DbPool;

use sha2::Sha256;

use crate::models::{NewPost, Post, PostPayload};
use crate::TokenClaims;
use actix_web::{
    delete, get, http::header::LOCATION, post, put, web, web::ReqData, Error, HttpResponse,
    Responder,
};
use actix_web_httpauth::extractors::basic::BasicAuth;
use argonautica::{Hasher, Verifier};
use diesel::prelude::*;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use log::{info, warn};
use rusty_api::models::Users;

type DbError = Box<dyn std::error::Error + Send + Sync>;

#[post("/users")]
async fn create_user(
    pool: web::Data<DbPool>,
    payload: web::Json<crate::models::UserPayload>,
) -> Result<HttpResponse, Error> {
    info!("entered create_user");
    let user = web::block(move || {
        let mut conn = pool.get()?;
        create_helper_fn(&mut conn, &payload.username, &payload.password)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(user))
}

#[post("/auth")]
async fn basic_auth(
    pool: web::Data<DbPool>,
    credentials: BasicAuth,
) -> Result<HttpResponse, Error> {
    let jwt_secret: Hmac<Sha256> = Hmac::new_from_slice(
        std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set!")
            .as_bytes(),
    )
    .unwrap();

    let password = credentials.password();
    let username = credentials.user_id().to_owned();

    info!("username: {}", username);
    let desired_user = web::block(move || {
        let mut conn = pool.get()?;
        find_by_username(username, &mut conn)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    info!("desired_user: {:?}", desired_user);
    match password {
        None => Ok(HttpResponse::Unauthorized().json("Must provide username and password")),
        Some(pass) => {
            let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set!");
            info!("hash_secret: {}", hash_secret);
            let mut verifier = Verifier::default();
            let is_valid = verifier
                .with_hash(desired_user.password)
                .with_password(pass)
                .with_secret_key(hash_secret)
                .verify()
                .unwrap();

            if is_valid {
                let claims = TokenClaims {
                    id: desired_user.id,
                };
                let token_str = claims.sign_with_key(&jwt_secret).unwrap();
                info!("token_str: {}", token_str);
                Ok(HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/admin"))
                    .finish())
            } else {
                Ok(HttpResponse::Unauthorized().json("Incorrect username or password"))
            }
        }
    }
}

#[get("/posts")]
async fn get_posts(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let posts = web::block(move || {
        let mut conn = pool.get()?;
        get_all_posts(&mut conn)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(posts))
}

#[post("/posts")]
async fn create(
    pool: web::Data<DbPool>,
    payload: web::Json<PostPayload>,
    req_user: Option<ReqData<crate::TokenClaims>>,
) -> Result<HttpResponse, Error> {
    match req_user {
        Some(user) => {
            let post = web::block(move || {
                let mut conn = pool.get()?;
                create_post(&mut conn, &payload.title, &payload.message)
            })
            .await?
            .map_err(actix_web::error::ErrorInternalServerError)?;
            Ok(HttpResponse::Ok().json(post))
        }
        None => {
            warn!("no user");
            Ok(HttpResponse::Unauthorized().json("Must provide a valid token"))
        }
    }
}

#[get("/posts/{id}")]
async fn show(_id: web::Path<i32>, pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let post = web::block(move || {
        let mut conn = pool.get()?;
        find_by_id(_id.into_inner(), &mut conn)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(post))
}

#[put("/posts/{id}")]
async fn update(_id: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("post#edit {}", _id))
}

#[delete("/posts/{id}")]
async fn destroy(_id: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("post#delete {}", _id))
}

fn create_helper_fn(
    conn: &mut PgConnection,
    _username: &str,
    _password: &str,
) -> Result<Users, DbError> {
    use crate::schema::users::dsl::*;

    let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set!");
    let mut hasher = Hasher::default();
    info!("hash_secret: {}", hash_secret);

    let hash = hasher
        .with_password(_password)
        .with_secret_key(hash_secret)
        .hash()
        .unwrap();

    let new_user = crate::models::NewUser {
        username: _username,
        password: &hash,
    };

    let res = diesel::insert_into(users)
        .values(&new_user)
        .get_result(conn)?;

    info!("user created");
    Ok(res)
}

pub fn create_post(conn: &mut PgConnection, _title: &str, _body: &str) -> Result<Post, DbError> {
    use crate::schema::posts::dsl::*;
    let new_post = NewPost {
        title: _title,
        body: _body,
    };

    let res = diesel::insert_into(posts)
        .values(&new_post)
        .get_result(conn)?;

    Ok(res)
}

fn find_by_id(post_id: i32, conn: &mut PgConnection) -> Result<Option<Post>, DbError> {
    use crate::schema::posts::dsl::*;
    let post = posts
        .filter(id.eq(post_id))
        .first::<Post>(conn)
        .optional()?;

    Ok(post)
}

fn find_by_username(_username: String, conn: &mut PgConnection) -> Result<Users, DbError> {
    use crate::schema::users::dsl::*;
    let user = users.filter(username.eq(_username)).first::<Users>(conn)?;

    Ok(user)
}

fn get_all_posts(conn: &mut PgConnection) -> Result<Vec<Post>, DbError> {
    use crate::schema::posts::dsl::*;

    let all_posts = posts.load::<Post>(conn).expect("Error loading posts");

    Ok(all_posts)
}
