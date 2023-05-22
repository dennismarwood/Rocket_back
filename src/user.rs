use rocket::serde::json::{Json, Value, json};
use diesel::prelude::*;
use crate::config::DbConn;
use crate::models::{NewUser, User, AResponse};
use crate::schema::{user, role};
use crate::pw::get_phc;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::status;
use rocket::Request;
//use rocket::form::Form;
use crate::myjsonapi::{JSONAPIError,};
use rocket::State;
use crate::models::EnvVariables;
//use rocket::response::Redirect;
//use crate::index::home;
//#[macro_use] extern crate serde_derive;

pub mod routes {
    use crate::{auth::{Level1, ValidSession}, jwt::get_jwt};
    use super::*;

    #[catch(422)]
    pub fn dup_entry() -> Value {
        json!(format!("Ensure email is unique and role is valid."))
    }

    #[catch(default)]
    pub fn catch_all(status: Status, _req: &Request) -> Value {
        let message = &mut JSONAPIError{
            status: status.code.to_string(), 
            canonical: String::from(status.reason().unwrap()), 
            title: String::from(""),
            detail: String::from("")};
        
        match status.code {
            401 => {
                message.title = String::from("Session token missing or invalid.");
                message.detail = String::from("The JWT is not present or is no longer valid.");
            }
            _ => {
                message.title = String::from("A error was generated but it had an unantacipated code.");
                message.detail = String::from("The back end of the application generated an unexpected non 200 response.");
            }
        }
        
        json!({"errors": vec![message]})
    }

    #[derive(serde::Deserialize, Clone)]
    pub struct Login {
        email: String,
        password: String,
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
 
    #[derive(Debug, FromForm, serde::Deserialize, AsChangeset)]
    #[diesel(table_name = user)]
    pub struct UpdateUser {
        pub email: Option<String>,
        #[field(name = uncased("pass"))]
        pub phc: Option<String>, // Must be labled phc for diesel but this will initially be the user's new password
        pub first_name: Option<String>,
        pub last_name: Option<String>,
        pub role: Option<i32>,
        pub active: Option<bool>,
    }

    /* #[patch("/", format="form", data="<updated_user>")]
    pub async fn _patch_user_form(updated_user: Form<UpdateUser>) -> Value {
        /* 
        Data could be sent into the backend with key value tuples described here:https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/POST
        Reqwests does send data in that format if specified: https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.form 
        However, I want this backend to only respond to application/json type reqeusts.
        Keeping this in place as an exemple of a way this (or the frontend) may want to pass data around to itself.
        */
        //println!("patch_user not implimented for the content-type:application/x-www-form-urlencoded. {:?}", updated_user);
        todo!("\n\npatch_user not implimented for the content-type:application/x-www-form-urlencoded. {:?}", updated_user)
    } */

    #[patch("/<id>", format = "json", data="<updated_user>")]
    pub async fn update_user(id: i32, conn: DbConn, mut updated_user: Json<UpdateUser>, _x: Level1) -> Result<status::NoContent, status::Custom<Json<AResponse>>> {
        /*
            A user exists that can reset anyone's pw and other user setting. The guard ensure this is a "Level1" user.
            If other users are added in the future, or just another user of "Dennis" that can just publish blogs, then it would be a good idea to include 
            another patch method that does not take a user id and is limited to updating just the user who's id is found from their jwt (i.e. themsleves).
         */
        //If a new pw was sent, calculate phc first.
        println!("{:?}", updated_user);
        if updated_user.phc.is_some() {
            let pass = updated_user.phc.clone().unwrap();
            match get_phc(pass) {
                Ok(user_phc) => updated_user.phc = Some(user_phc),
                Err(_) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))), //There was a problem updating the user's pw.
            }
        };

        match conn.run(move |c| {
            diesel::update(user::table)
            .filter(user::id.eq(id))
            .set(updated_user.into_inner())
            .execute(c)
        }).await {
        Ok(_) => return Ok(status::NoContent),
        Err(_) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))), //There was a problem updating the user.
      }  
    } 
    
    #[post("/confirm_pw", format = "text", data="<form_pw>")]
    pub async fn confirm_pw(form_pw: String, conn:DbConn, user_id: ValidSession, _x: Level1) -> Result<Status, status::Custom<Json<AResponse>>> {
        //Whatever a user passes in as data is interpreted as a pw value.
        //A user must have a session (ValidSession guard).
        //Using the session user_id, check if pw is valid.
        //Get user's phc
        let user_phc = match
            conn.run( move |conn| {
                user::table
                .select(user::phc)
                .filter(user::id.eq(user_id.id))
                .first::<Option<String>>(conn)
            }).await
            {
                Ok(u_phc) => u_phc,
                Err(_) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))) //User has session but cannot be found in db?!
                //Err(_) => return Err(status::Custom(Status::Unauthorized, Json(AResponse::_401(Some(String::from("Provided password did not match user's current pw."))))))
            };
            println!("form_pw: {:?}", form_pw);
        match crate::pw::verify_password(&form_pw, &user_phc.unwrap_or_default()) {
                Ok(_) => {println!("Sending 200");return Ok(Status::Ok)}, //User provided pw is valid.
                Err(_) => {println!("Sending 401");return Err(status::Custom(Status::Unauthorized, Json(AResponse::_401(Some(String::from("Existing password was invalid."))))))}, //Provided pw was invalid
            }
            
    }

    #[get("/")]
    pub async fn get_user(conn:DbConn, user_id: ValidSession, _x: Level1) -> Result<Json<AResponse>, status::Custom<Json<AResponse>>> {
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
                
                return Ok(Json(AResponse::_200(Some(json!(entry)))))
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))), //There was a problem retrieving the user.
        };
    }

    #[post("/", format = "json", data="<new_user>")]//
    pub async fn add_user(conn: DbConn, new_user: Json<CreateNewUser>, _x: Level1) -> Result<status::Created<String>, status::Custom<Json<AResponse>>> {
        //TODO check that pass meets minimum criteria (length, uppper, number, etc)
        //TODO verify that email is valid format

        let mut user = NewUser {

            email: new_user.email.clone(),
            phc: None,
            first_name: new_user.first_name.clone(),
            last_name: new_user.last_name.clone(),
            //---------------------------------------------------------------------------------------------------
            //Careful here, allowing front end user to dicatate role level! Ok for now as only an admin user can add users.
            //--------------------------------------------------------------------------------------------------- 
            role: new_user.role,
            active: true,
        };

        match get_phc(new_user.pass.clone()) {
            Ok(user_phc) => user.phc = Some(user_phc),
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))), //There was a problem creating the phc.
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
            Ok(_) => Ok(status::Created::new(String::new())),
            Err(e) => Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                Some(e.to_string()), 
                Some(String::from("INVALID_FIELD")),
                None)))),
            }

    }

    #[delete("/<id>")]
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
                        json!(format!("Should have only deleted up to 1 record, BUT DELETED {}!!!", count)))),
                }
            },
            Err(e) => return Err(status::Custom(Status::InternalServerError, 
                json!(format!("Diesel returned an error: {}", e))))
        }
    }

/*     #[post("/session/update_pw", format = "json", data="<login>")]
    pub async fn validate_pw(conn: DbConn, credentials: Json<Login>) -> Option<status::Custom<Json<AResponse>>> {
        //To update a pw, the existing pw must be verified.
        match crate::pw::verify_password(&login.password, &user.phc.clone().unwrap_or_default()) {
            Ok(_) => (user, role), //provided email and pw are good
            Err(_) => return Err(status::Custom(Status::Unauthorized, Json(AResponse::_401(Some(String::from("Provided email or password was invalid.")))))), //Provided pw was invalid
        },
    }
 */

    #[post("/session", format = "json", data="<login>")]
    pub async fn start_session(conn: DbConn, login: Json<Login>, jar: &CookieJar<'_>, server_env_vars: &State<EnvVariables>) -> Result<Status, status::Custom<Json<AResponse>>> {
        let email_clone = login.email.clone();
        let (user, role) = match //Retrieve a user object and the user objects corresponding user_role
            conn.run( move |conn| {
                user::table
                .left_join(role::table)
                .select((User::as_select(), role::user_role.nullable()))
                .filter(user::email.eq(email_clone))
                .first::<(User, Option<String>)>(conn)
            }).await
        {
            Ok((user, role)) =>       
                match crate::pw::verify_password(&login.password, &user.phc.clone().unwrap_or_default()) {
                    Ok(_) => (user, role), //provided email and pw are good
                    Err(_) => return Err(status::Custom(Status::Unauthorized, Json(AResponse::_401(Some(String::from("Provided email or password was invalid.")))))), //Provided pw was invalid
                },
            Err(_) => 
                return Err(status::Custom(Status::Unauthorized, Json(AResponse::_401(Some(String::from("Provided email or password was invalid.")))))), //Provided email was invalid
        };

        match get_jwt(&user, role.unwrap().as_str(), server_env_vars.jwt_secret.as_ref()) {
            Ok(jwt) => 
            {
                let mut cookie = Cookie::new("jwt", jwt.clone());
                cookie.set_http_only(true);
                jar.add(cookie); //jwt added to client cookies

                match conn.run( move |conn| { //Update the last access column for the user
                    diesel::update(&user).set(user::last_access.eq(chrono::Utc::now().date_naive())).execute(conn)
                    }).await
                {
                    Ok(_) => return Ok(Status::Ok), // return 200
                    Err(_) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))), //There was a problem updating the last access column.
                };
            },
            Err(_) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))), //There was a problem creating the jwt.
        }

    }

    #[delete("/session")]
    pub async fn end_session(jar: &CookieJar<'_>, _x: Level1) -> Status {
        jar.remove(Cookie::named("jwt"));
        Status::Ok 
    }

}