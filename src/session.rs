use rocket::serde::json::{Value, json};//Json
//use diesel::prelude::*;
//use crate::config::DbConn;
//use crate::schema::{user, role};
//use crate::pw::verify_password;
//use crate::jwt::get_jwt;
use rocket::{Request};
use rocket::http::{Cookie, CookieJar, Status};
//use crate::models::EnvVariables;
//use rocket::State;

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

    /* #[post("/", format = "json", data="<credentials>")]
    //pub async fn session(conn: DbConn, credentials: Json<Credentials>, jar: &CookieJar<'_>) -> Result<Value, status::Custom<Value>> {
    pub async fn create_session(conn: DbConn, credentials: Json<Credentials>, jar: &CookieJar<'_>, my_config: &State<EnvVariables>) -> (Status, Value) { 
        let user_email = credentials.email.clone();
        let user_pass = credentials.pass.clone();
        let user_data = match 
            conn.run( |c| {
                user::table
                .left_join(role::table)
                .filter(user::email.eq(user_email))
                .select((user::id, user::phc, user::role, role::user_role.nullable()))
                .first::<(i32, Option<String>, i32, Option<String>)>(c)//Throw error if 0 rows returned
            }).await
            {
                Ok(x) => x,
                Err(_) => {
                    //.map_err(|_| status::Custom(Status::Unauthorized, json!("Invalid email and/or password.")))?;
                    return (Status::Unauthorized, json!({
                        "errors": [
                            {
                                "status": 401,
                                "description": "The provided email/password combination is invalid or no user with that email exists."
                            }
                        ]
                    }))
                }
            };
        
        match verify_password(&user_pass, &user_data.1.unwrap()){
            Ok(_) =>{},
            Err(_) => return (Status::Unauthorized, json!({//A valid email was provided but an incorrect pw.
                "errors": [
                    {
                        "status": 401,
                        "description": "The provided email/password combination is invalid or no user with that email exists."
                    }
                ]
            }))
        };

        let user_email = credentials.email.clone();

        //User is logging in, update "last_access" column in their row.
        match conn.run(|c| {
            diesel::update(user::table)
            .filter(user::email.eq(user_email))
            .set(user::last_access.eq(Some(chrono::Utc::now().date_naive())))
            .execute(c)
        }).await {
            Ok(_) => {},
            Err(e) => return (Status::InternalServerError, json!({
                "errors": [
                    {
                        "status": 500,
                        "description": format!("{}", e)
                    }
                ]
            }))
        }

        //The user email / password combo was valid. Issue them the appropriate jwt.
        match get_jwt(
            user_data.0,
            credentials.email.clone(),
            user_data.2,
            user_data.3.unwrap(),
            my_config.jwt_secret.as_ref())
            {
                Ok(jwt) => {
                    let mut cookie = Cookie::new("jwt", jwt.clone());
                    cookie.set_http_only(true);
                    jar.add(cookie);
                    return (Status::Ok, json!({
                        "information": [
                            {
                                "status": 200,
                                "description": "A jwt for the user was generated."
                            }
                        ]
                    }))
                }
                Err(e) => return (Status::InternalServerError, json!({
                    "errors": [
                        {
                            "status": 500,
                            "description": format!("{}", e)
                        }
                    ]
                }))
            }
    } */

    #[delete("/", format = "json")]
    pub async fn destroy_session(jar: &CookieJar<'_>) -> Status {
        //Reminder that removing the cookie from the client is a session destruction half measure. A duplicate of the deleted jwt would still 
        //be validated by the guard.
        jar.remove(Cookie::named("jwt"));
        //Redirect::to(uri!(home));//Front end should redirect?
        Status::Ok 
    }
}