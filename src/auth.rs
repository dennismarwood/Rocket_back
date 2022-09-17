use rocket::http::Status;
use rocket::request::{Request, FromRequest, Outcome};
use crate::jwt::{validate_jwt};
use crate::models::EnvVariables;

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
                            _ => Outcome::Failure((Status::Unauthorized, ())),
                        }     
                    },
                    Err(_) => Outcome::Failure((Status::Unauthorized, ())),
                }
            },
            None => return Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}