#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
use rocket::fairing::AdHoc;

mod models;
mod schema;
mod auth;
mod pw;
mod jwt;
mod blog_tags;
mod myjsonapi;

mod api;
use api::*;

mod index;
use index::home;

mod config;
use config::*;

mod post;
use post::routes::*;

mod tag;
use tag::routes::*;

mod user;
use user::routes::*;

mod role;
use role::routes::*;

mod session;
use session::routes::*;

use models::EnvVariables;

use rocket_cors::{AllowedHeaders, AllowedOrigins};
use rocket::http::Method;

#[launch]
fn rocket() -> _ {
    //let allowed_origins = AllowedOrigins::all();
    let allowed_origins = AllowedOrigins::some_regex(&["^vscode-webview://(.+)"]);
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors().unwrap();

    rocket::build()
        .mount("/", routes![home])
        .mount("/api", routes![api_info])
        .mount("/api/tags", routes![
            get_tags,
            get_tag_by_id,
            get_tag_by_name,
            get_tags_on_post,
            add_tag
        ])
        .mount("/api/posts", routes![
            get_post_by_id, 
            get_post_by_title,
            get_post_by_author, 
            get_post_by_tags,
            get_post_by_tags_from_to,
            get_post_by_from_to,
            update_post,
            new_post
        ])
        .mount("/api/roles", routes![
            get_roles,
            get_role,
            new_role,
            update_role,
            delete_role
        ])
        .mount("/session", routes![
            create_session, 
            destroy_session
        ])
            .register("/session", catchers![
                email_or_pw_incorrect
            ])
        .mount("/user", routes![
            add_user, 
            delete_user, 
            update_user,
            get_user,
            patch_user
        ])
            .register("/user", catchers![
                dup_entry,
                catch_all
            ])
        .attach(DbConn::fairing())
        .attach(AdHoc::config::<EnvVariables>())
        .attach(cors)
}