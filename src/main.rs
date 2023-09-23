#[macro_use]
extern crate diesel;

use actix_cors::Cors;
use actix_files as fs;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::HttpMessage;
use actix_web::{
    cookie::Key, dev::ServiceRequest, http::header, middleware, web, App, Error, HttpResponse,
    HttpServer,
};
use actix_web_httpauth::{
    extractors::{
        bearer::{self, BearerAuth},
        AuthenticationError,
    },
    middleware::HttpAuthentication,
};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

mod errors;
mod handlers;
mod models;
mod schema;

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    id: i32,
}

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let key: Hmac<Sha256> =
        Hmac::new_from_slice(jwt_secret.as_bytes()).expect("HMAC can take key of any size");
    let token_string = credentials.token();

    let claims: Result<TokenClaims, &str> = token_string
        .verify_with_key(&key)
        .map_err(|_| "Invalid token");

    match claims {
        Ok(value) => {
            req.extensions_mut().insert(value);
            Ok(req)
        }
        Err(_) => {
            let config = req
                .app_data::<bearer::Config>()
                .cloned()
                .unwrap_or_default()
                .scope("");

            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

async fn fallback_route() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("frontend/index.html")?)
}

async fn login_route() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("frontend/login.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Loading .env into environment variable.
    dotenv::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // set up database connection pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let redis_uri = std::env::var("REDIS_URI").expect("REDIS_URI must be set");
    let secret_key = Key::generate();
    let redis_session_store = RedisSessionStore::new(redis_uri).await.unwrap();

    HttpServer::new(move || {
        let bearer_middleware = HttpAuthentication::bearer(validator);
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(SessionMiddleware::new(
                redis_session_store.clone(),
                secret_key.clone(),
            ))
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .service(handlers::get_posts)
                    .service(handlers::show)
                    .service(handlers::update)
                    .service(handlers::destroy)
                    .service(handlers::create_user)
                    .service(handlers::basic_auth)
                    .service(
                        web::scope("")
                            .wrap(bearer_middleware)
                            .service(handlers::create),
                    ),
            )
            .service(fs::Files::new("/frontend", "./frontend").show_files_listing())
            .route("/login", web::get().to(login_route))
            .route("/home", web::get().to(fallback_route))
    })
    .bind(("127.0.0.1", 3030))?
    .run()
    .await
}
