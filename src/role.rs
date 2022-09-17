use rocket::serde::json::{Json, Value, json};
use diesel::prelude::*;
use crate::config::DbConn;
use crate::models::Role;
use crate::schema::{role};
use rocket::http::{Status};
use rocket::response::status;

pub mod routes {
    use super::*;
    use crate::auth::Level1;

    #[get("/")]
    pub async fn get_roles(conn: DbConn, _x: Level1) -> Result< Value, status::Custom<Value>> {
        match conn.run(|c| {
            role::table
                .limit(100)
                .load::<Role>(c)
        }).await
        {
            Ok(results) => Ok(json!(results)),
            Err(e) => Err(status::Custom(Status::InternalServerError , json!(format!("{}", e)))),
        }
    }

    #[get("/<id>")]
    pub async fn get_role(conn: DbConn, id: i32, _x: Level1) -> Result< Value, status::Custom<Value>> {
        match conn.run(move |c| {
            role::table
                .filter(role::id.eq(id))
                .load::<Role>(c)
        }).await
        {
            Ok(results) => Ok(json!(results)),
            Err(e) => Err(status::Custom(Status::InternalServerError , json!(format!("{}", e)))),
        }
    }

    #[post("/", data = "<new_entry>")]
    pub async fn new_role(conn: DbConn, new_entry: Json<Role>, _x: Level1) -> Result< Value, status::Custom<Value>> {
        println!("{:?}", new_entry);
        match conn.run(move |c| {
        diesel::insert_into(role::table)
            .values((
                role::id.eq(new_entry.id.clone()),
                role::user_role.eq(new_entry.user_role.clone())
            ))
            .execute(c)
        }).await
        {
            Ok(results) => Ok(json!(results)),
            Err(e) => Err(status::Custom(Status::InternalServerError , json!(format!("{}", e)))),
        }

    }

    #[post("/<id>", data = "<new_entry>")]
    pub async fn update_role(conn: DbConn, id: i32, new_entry: Json<Role>, _x: Level1) -> Result< Value, status::Custom<Value>> {
        match conn.run(move |c| {
            diesel::update(role::table)
                .filter(role::id.eq(id))
                .set(
                    (
                                role::id.eq(new_entry.id),
                                role::user_role.eq(new_entry.user_role.clone())
                            )
                    )
                .execute(c)
        }).await
        {
            Ok(results) => Ok(json!(results)),
            Err(e) => Err(status::Custom(Status::InternalServerError , json!(format!("{}", e)))),
        }
    }

    #[post("/<id>/delete")]
    pub async fn delete_role(conn: DbConn, id: i32, _x: Level1) -> Result< Value, status::Custom<Value>> {
        match conn.run(move |c| {
            diesel::delete(
                role::table
                .filter(role::id.eq(id))
            )
            .execute(c)
        }).await
        {
            Ok(results) => Ok(json!(results)),
            Err(e) => Err(status::Custom(Status::InternalServerError , json!(format!("{}", e)))),
        }
    }
    
}