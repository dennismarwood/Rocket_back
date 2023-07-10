#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
use rocket::fairing::AdHoc;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use rocket::http::Method;
use rocket::fs::{FileServer, NamedFile, relative};

//use rocket_contrib::serve::StaticFiles;
mod models;
mod schema;
mod auth;
mod pw;
mod jwt;
mod post_tags;
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

#[get("/openapi_yml")]
async fn openapi_yml() -> Option<NamedFile> {
    //https://github.com/swagger-api/swagger-ui/blob/master/docs/usage/installation.md#plain-old-htmlcssjs-standalone
    NamedFile::open("src/homepage.yml").await.ok()
}

/* #[get("/openapi_index")]
async fn openapi_index() -> Option<NamedFile> {
    NamedFile::open("static/3rd_party/swagger-ui-4.18.1/dist/index.html").await.ok()
} */

#[launch]
fn rocket() -> _ {
    //let allowed_origins = AllowedOrigins::all();
    let allowed_origins = AllowedOrigins::some_regex(&["^vscode-webview://(.+)"]);
    //let allowed_origins = AllowedOrigins::some_null();
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors().unwrap();


    rocket::build()
        //.mount("/my_path", StaticFiles::from("/www/public"))
        //.mount("/openapi", FileServer::from("/static/3rd_party/swagger-ui-4.18.1/dist"))
        //https://github.com/swagger-api/swagger-ui/releases
        .mount("/openapi", FileServer::from(relative!("/static/3rd_party/swagger-ui-4.19.0/dist")))
        .mount("/", routes![home, openapi_yml])
        .mount("/api", routes![api_info])
        .mount("/api/tags", routes![
            tag::routes::get,
            get_tags,
            tag::routes::patch,
            tag::routes::post,
            tag::routes::delete,
            tag::routes::get_posts,
        ])
        .mount("/api/posts", routes![
            post::routes::get_posts,
            post::routes::get,
            post::routes::post_,
            post::routes::patch,
            post::routes::delete,
            put_post_tag,
            patch_post_tags,
            patch_post_tags_form,
            put_post_tags_form,
            delete_post_tag
        ])
        .mount("/api/roles", routes![
            get_roles,
            get_role,
            new_role,
            update_role,
            delete_role
        ])
        /* .mount("/session", routes![
            create_session, 
            destroy_session
        ]) */
            .register("/api/session", catchers![
                email_or_pw_incorrect
            ])
        .mount("/api/users", routes![
            add_user, 
            delete_user,
            delete_user_forbidden,
            delete_user_unauthorized,
            update_user,
            update_user_forbidden,
            update_user_unauthorized,
            update_self,
            get_user_unauthorized,
            get_user,
            get_user_admin,
            start_session,
            end_session,
            confirm_pw,
            list_of_all_users,
            list_of_all_users_forbidden,
            list_of_all_users_unauthorized,
            get_user_by_id,
            get_user_by_id_unauthorized,
            get_user_by_id_forbidden,
            //patch_user
        ])
            .register("/user", catchers![
                dup_entry,
                catch_all
            ])
        .attach(DbConn::fairing())
        .attach(AdHoc::config::<EnvVariables>())
        .attach(cors)
}