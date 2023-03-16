use rocket::http::Status;
use rocket::request::{Request, FromRequest, Outcome};
use crate::jwt::{validate_jwt};
use crate::models::EnvVariables;
use rocket::serde::json::{Json, Value, json};
use rocket::response::status;
use crate::myjsonapi::JSONAPIError;


/* 
    pub struct BasicAuth {
        pub username: String,
        pub password: String,
    }

    impl BasicAuth {
        fn from_authorization_header(header: &str) -> Option<BasicAuth> {
            let split = header.split_whitespace().collect::<Vec<_>>();
            if split.len() != 2 {
                return None;
            }

            if split[0] != "Basic" {
                return None;
            }

            Self::from_base64_encoded(split[1])
        }

        fn from_base64_encoded(base64_string: &str) -> Option<BasicAuth> {
            let decoded = base64::decode(base64_string).ok()?; //Return None if ok gets None from decode
            let decoded_str = String::from_utf8(decoded).ok()?;
            let split = decoded_str.split(":").collect::<Vec<_>>();

            if split.len() != 2 {
                return None;
            }

            let (username, password) = (split[0].to_string(), split[1].to_string());

            Some(BasicAuth {
                username,
                password
            })
        }
    } 

    #[rocket::async_trait]
    impl<'r> FromRequest <'r> for BasicAuth {
        type Error = ();

        async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
            let auth_header = request.headers().get_one("Authorization");
            if let Some(auth_header) = auth_header {
                if let Some(auth) = Self::from_authorization_header(auth_header) {
                    return Outcome::Success(auth)
                }
            }
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }*/

pub struct Level1 {
    pub role_id: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest <'r> for Level1 {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Level1, Self::Error> {
        let secret = request.rocket().state::<EnvVariables>().unwrap().jwt_secret.clone();
        
        match request.cookies().get("jwt") 
        {
            Some(unvalidated_jwt) => {
                match validate_jwt(unvalidated_jwt.value(), secret.as_ref()) 
                {
                    Ok(claims) => {
                        match claims.role_id {
                            1 => Outcome::Success(Level1{role_id: 1}),
                            _ => Outcome::Failure((Status::Forbidden, ())), //Has JWT, but JWT is for a user with incorrect role privileges
                        }     
                    },
                    Err(_) => Outcome::Failure((Status::Unauthorized, ())), //JWT invalid, probably expired
                }
            },
            None => return Outcome::Failure((Status::Unauthorized, ())), //Had no JWT
        }
    }
}

pub struct ValidSession{
    pub id: i32,
}

/*
What I wanted to do here was to specify the data that a Outcome::Failure returns.
Unfortunately, that is not possible.
There is a way to prevent duplicate runs of expensive / slow operations.
https://api.rocket.rs/v0.4/rocket/request/trait.FromRequest.html#request-local-state
We can also use this as a way to pass data back from the guard to a catcher.
We can load a data type into the local cache. The first load of a data type is not replaced.
Lets load up some json into the local cache.

let s = Status::Unauthorized;
let message = json!(&JSONAPIError{
    status: s.code.to_string(), 
    canonical: String::from(s.reason().unwrap()), 
    title: String::from("Session token missing or invalid."),
    detail: String::from("The JWT is not present or is no longer valid.")});
request.local_cache(|| message);

Then in a catcher we can pull that data out...
(from user.rs)
#[catch(default)]
    pub fn catch_all(status: Status, req: &Request) -> Value {
        json!(format!("{:?}", req.local_cache(|| String::from("ERROR"))));
    }

However, there is no data that is generated in this guard that cannot be just written up
in the catcher. See the entries in JSONAPIError above.
Even though I can send data back to the catcher there is no need to do so in this case.
*/
#[rocket::async_trait]
impl<'r> FromRequest <'r> for ValidSession {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<ValidSession, Self::Error> {// MyError<Value>> { 
        let secret = request.rocket().state::<EnvVariables>().unwrap().jwt_secret.clone();
        match request.cookies().get("jwt") 
        {
            Some(unvalidated_jwt) => {
                match validate_jwt(unvalidated_jwt.value(), secret.as_ref()) 
                {
                    Ok(claims) => Outcome::Success(ValidSession{id: claims.user_id}),
                    Err(_) => Outcome::Failure((Status::Unauthorized, ())), //JWT is present but invalid, probably expired
                }
            },
            None => Outcome::Failure((Status::Unauthorized, ())), //Had no JWT
        }
    }
}