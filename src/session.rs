use rocket::serde::json::{Json, Value, json};
use diesel::prelude::*;
use crate::config::DbConn;
use crate::schema::{user, role};
use crate::pw::verify_password;
use crate::jwt::get_jwt;
use rocket::{Request};
use rocket::http::{Cookie, CookieJar, Status};
use crate::models::EnvVariables;
use rocket::State;

pub mod routes {

    use super::*;

    #[catch(401)]
    pub fn email_or_pw_incorrect(req: &Request<'_>) -> Value {
        json!(format!("You lack needed persmisison or your provided credentials were incorrect. {}", req.uri()))
    }

    #[derive(serde::Deserialize)]
    pub struct Credentials {
        pub email: String,
        pub pass: String,
        pub phc: Option<String>,
    }

    #[post("/", format = "json", data="<credentials>")]
    //pub async fn session(conn: DbConn, credentials: Json<Credentials>, jar: &CookieJar<'_>) -> Result<Value, status::Custom<Value>> {
    pub async fn create_session(conn: DbConn, credentials: Json<Credentials>, jar: &CookieJar<'_>, my_config: &State<EnvVariables>) -> Result<Status, Status> { 
        let user_email = credentials.email.clone();        
        let user_data = conn.run( |c| {
            user::table
            .left_join(role::table)
            .filter(user::email.eq(user_email))
            .select((user::phc, user::role, role::user_role.nullable()))
            .first::<(Option<String>, i32, Option<String>)>(c)
        }).await
        //.map_err(|_| status::Custom(Status::Unauthorized, json!("Invalid email and/or password.")))?;
        .map_err(|_| Status::Unauthorized)?;

        verify_password(credentials.pass.clone(), user_data.0.unwrap())
        .map_err(|_| return Status::Unauthorized)?;

        let user_email = credentials.email.clone();
        match conn.run(|c| {
            diesel::update(user::table)
            .filter(user::email.eq(user_email))
            .set(user::last_access.eq(Some(chrono::Utc::now().date_naive())))
            .execute(c)
        }).await {
            Ok(_) => {},
            Err(_) => return Err(Status::InternalServerError)
        }

        
        match get_jwt(
            credentials.email.clone(),
            user_data.1,
            user_data.2.unwrap(),
            my_config.jwt_secret.as_ref()){
            Ok(jwt) => {
                let mut cookie = Cookie::new("jwt", jwt.clone());
                cookie.set_http_only(true);
                jar.add(cookie);
                Ok(Status::Ok)
            }
            Err(e) => {
                println!("\n{}", e);
                Err(Status::InternalServerError)
            }
        }
      
    }

    #[delete("/", format = "json")]
    pub async fn destroy_session(jar: &CookieJar<'_>) -> Status {
        //Reminder that removing the cookie from the client is a session destruction half measure. A duplicate of the deleted jwt would still 
        //be validated by the guard.
        jar.remove(Cookie::named("jwt"));
        //Redirect::to(uri!(home));//Front end should redirect?
        Status::Ok 
    }
}