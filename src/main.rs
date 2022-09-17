#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
use rocket::fairing::AdHoc;

mod models;
mod schema;
mod auth;
mod pw;
mod jwt;
mod blog_tags;

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
#[launch]
fn rocket() -> _ {
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
            update_user
        ])
            .register("/user", catchers![
                dup_entry
            ])
        .attach(DbConn::fairing())
        .attach(AdHoc::config::<EnvVariables>())
}