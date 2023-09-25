#[macro_use]
extern crate diesel;

use crate::auth::validator;
use actix_cors::Cors;
use actix_files as fs;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

mod auth;
mod handlers;
mod models;
mod schema;

async fn fallback_route() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("frontend/index.html")?)
}

async fn login_route() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("frontend/login.html")?)
}

async fn admin_route() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("frontend/admin.html")?)
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
            .route("/admin", web::get().to(admin_route))
    })
    .bind(("127.0.0.1", 3030))?
    .run()
    .await
}
