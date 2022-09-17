use jsonwebtoken::*;
use chrono::{Utc, Duration};
use crate::models::JWTClaims;

/*
CSRF - Originally, the default behavior of a browser was to include all cookies it had to every website you went to.
Imagine you have two tabs open. One is BofA (and you are logged in there), the other is malicious site x.
When you navigate to x you will be passing your login credintials for BofA along to x.
Then x can just use your credentials to post to BofA passing along your credentials. BofA thinks x is you.
Now, browsers will default to a stricter level of security for cookies. This is called "same-site".
Three tiers of same-site exist.
    None: The behavior described above, send all cookies to all websites.
    Lax: The, now, default behavior. Only send cookies along from "third party" (x in our example) if the URI is of type GET.
        Note that Iframes are also "third party". So are amazon links.
    Strict: Only send along cookies if the target site is of the same origin as the requesting site.
Note that using private cookies in rust will also default the cookie to use the strict tier of restriction.

xss is when input "<" or ">" are not escaped and the user is able to execute js.

TODO: Upgrade to asymmetric signatures. Then move jws token generator to a seperate service and pass around public key to other services.
TODO: The ttl could be extended a bit and then we could check to see that a user is still active by updating a time since last check var.
      If the var is too great, assume the user is done and kill the session.
TODO: When a user logs out or changes password etc, it is not really enough to destroy the cookie in client. If we were keeping track of 
      tracking the activity of the user then we could also 0 out that time remaining to renew, effectively invalidating the jwt and 
      forcing an end to their session.
TODO: Key should be disgarded and regenerated every 24 hours.
      Kid (Key IDentifier) is a field that lets you track jwts. This could be used for passing out the appropriate pub key for 
      overlapping valid keys. For example; a key can live for 24 hours but you issue new ones every 12 hours.
*/

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Claims {
    email: String,
    role_id: i32,
    role: String,
    exp: usize
}


pub fn get_jwt(user_email: String, role_id: i32, user_role: String, secret: &[u8] ) -> Result<String, jsonwebtoken::errors::Error> {
    //let figment = rocket::figment::Figment::from(rocket::Config::default());
    //let secret = figment.extract_inner("jwt_secret").expect("ROCKET_JWT_SECRET env value was not found.");

    let ttl = Duration::hours(2);
    let expiration = Utc::now()
        .checked_add_signed(ttl)
        .expect("failed to make jwt expiration.")
        .timestamp(); //Returns the number of non-leap seconds since January 1, 1970 0:00:00 UTC (aka “UNIX timestamp”).
    let claims = Claims {
        email: user_email.clone(),
        role_id: role_id,
        role: user_role.clone(),
        exp: expiration as usize,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        //&EncodingKey::from_secret(&server_config.jwt_secret_key),
        &EncodingKey::from_secret(secret),
    )//.map(|s| s.clone())
    //.map_err(|e| e.to_string())
}

pub fn validate_jwt(jwt: &str, secret: &[u8]) -> Result< JWTClaims, jsonwebtoken::errors::Error> {
    //let figment = rocket::figment::Figment::from(rocket::Config::default());
    //let secret = figment.extract_inner("jwt_secret").expect("ROCKET_JWT_SECRET env value was not found.");

    let claims = decode::<JWTClaims>(
        jwt,
        &DecodingKey::from_secret(secret), 
        &Validation::new(Algorithm::HS256)
    );
    
    match claims {
        Ok(c) => {
            //c: TokenData { 
                //header: Header { 
                    //type: Some("JWT"), alg: HS256, cty: None, jku: None, jwk: None, kid: None, x5u: None, x5c: None, x5t: None, x5t_s256: None }, 
                //claims: JWTClaims { email: "mail@gmail.com", role_id: 1, role: "admin", exp: 1662178970 } }

            //return Outcome::Success(c.claims)
            //return Outcome::Success(JWTClaims {email:"email".to_string(), role_id:10, role:"role".to_string(), exp:9999})
            Ok(c.claims)
        },
        Err(e) => {
            Err(e)
            //println!("error: {}", e);
            //return Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}
