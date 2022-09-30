use rocket::serde::json::{Json, Value, json};
use diesel::prelude::*;
use crate::config::DbConn;
use crate::models::{NewUser, User};
use crate::schema::{user};
use crate::pw::get_phc;
use rocket::http::{Status};
use rocket::response::status;
use diesel::*;
//use rocket::response::Redirect;
//use crate::index::home;

pub mod routes {
    use crate::auth::{Level1, ValidSession};
    use super::*;

    #[catch(422)]
    pub fn dup_entry() -> Value {
        json!(format!("Ensure email is unique and role is valid."))
    }

    #[derive(serde::Deserialize)]
    pub struct CreateNewUser {
        pub email: String,
        pub pass: String,
        pub first_name: Option<String>,
        pub last_name: Option<String>,
        pub role: i32,
        pub active: bool,
    }

    #[derive(serde::Deserialize, AsChangeset)]
    #[table_name ="user"]
    pub struct UpdateUser {
        pub email : Option<String>,
        pub phc :Option<String>, // Must be labled phc for diesel but this will initially be the user's new password
        pub first_name: Option<String>,
        pub last_name: Option<String>,
        pub role: Option<i32>,
        pub active: Option<bool>,
    }

    //Because all fields are optional, it is expceted to use patch here.
    #[patch("/<id>", format = "json", data="<updated_user>")]
    pub async fn update_user(id: i32, conn: DbConn, mut updated_user: Json<UpdateUser>, _x: Level1) -> Result<Status, status::Custom<Value>> {
        //TODO: Updating a pw should invalidate this user's jwt. We should update user fields, invalidate jwt (not implemented), then give them a new jwt.
        //If a new pw was sent, calculate phc first.
        if updated_user.phc.is_some() {
            let pass = updated_user.phc.clone().unwrap();
            match get_phc(pass) {
                Ok(user_phc) => updated_user.phc = Some(user_phc),
                Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("Failed to updated the user's PW. {}", e)))),
            }
        };

        match conn.run(move |c| {
            diesel::update(user::table)
            .filter(user::id.eq(id))
            .set(updated_user.into_inner())
            .execute(c)
        }).await {
        Ok(_) => return Ok(Status::NoContent),
        Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("Failed to update the user. {}", e)))),
      }  
    } 
    
    #[get("/")]
    pub async fn get_user(conn:DbConn, user_id: ValidSession) -> (Status, Value) {
        match conn.run(move |c: &mut MysqlConnection| {
            user::table
                .filter(user::id.eq(user_id.id))
                .first::<User>(c)
            }).await
        {
            Ok(entry) => {
                //Convert user object into json. Convert that json into a serde_json map.
                //Remove an element from the map.
                //Convert the map back into json.
                let mut map: serde_json::Map<String, Value> = serde_json::from_value(json!(entry)).unwrap();
                map.remove("phc");
                let entry = json!(map);
                
                return (Status::Ok, entry)
            },
            Err(e) => {
                return (Status::InternalServerError, 
                    json!({
                        "errors": [
                            {
                                "status": 500,
                                "description": format!("{}", e)
                            }
                        ]
                    })
                )
            },
        };
        //todo!("shh")
    }

    #[post("/", format = "json", data="<new_user>")]
    pub async fn add_user(conn: DbConn, new_user: Json<CreateNewUser>, _x: Level1) -> Result<Status, Status> {
        //TODO check that pass meets minimum criteria (length, uppper, number, etc)
        //TODO verify that email is valid format

        let mut user = NewUser {

            email: new_user.email.clone(),
            phc: None,
            first_name: new_user.first_name.clone(),
            last_name: new_user.last_name.clone(),
            role: -1,
            active: true,
        };

        match get_phc(new_user.pass.clone()) {
            Ok(user_phc) => user.phc = Some(user_phc),
            Err(_) => return Err(Status::InternalServerError)
        }

        /* 
        match get_role_id(&conn, new_user.role.clone()).await {
            Ok(role_id) => user.role = role_id,
            Err(_) => { println!("here"); return Err(Status::UnprocessableEntity) }
        } */

        match conn.run(|c| {
                    diesel::insert_into(user::table)
                    .values(user)
                    .execute(c)
                }).await {
            Ok(_) => Ok(Status::Created),
            Err(_) => Err(Status::UnprocessableEntity),
            }

    }

    #[delete("/<id>", format = "json")]
    pub async fn delete_user(id: i32, conn: DbConn, _x: Level1) -> Result<Status, status::Custom<Value>> {
        match conn.run(move |c| {
            diesel::delete(user::table
                .filter(user::id.eq(id))
            )
            .execute(c)
            }).await
        {
            Ok(count) => {
                match count {
                    1 => return Ok(Status::NoContent),
                    0 => return Ok(Status::NotFound),
                    _ => return Err(status::Custom(Status::InternalServerError, 
                        json!(format!("Should have only deleted up to 1 record, BUT DELETED {}!!!", count)))),// Hope you backed up
                }
            },
            Err(e) => return Err(status::Custom(Status::InternalServerError, 
                json!(format!("Diesel returned an error: {}", e))))
        }
    }

}

/* pub mod helper {
    use super::*;

 
} */