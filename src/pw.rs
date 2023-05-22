use pbkdf2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Pbkdf2
};

    // store new phc in db
    // retrieve phc for user and verify

    pub fn verify_password(password: &String, phc: &String) -> Result<(), pbkdf2::password_hash::Error>{
        println!("password - {:?}\nph - {}", password, phc);
        let password = password.as_bytes();//User provided pw
        let parsed_hash = PasswordHash::new(&phc);//User's computed phc
        match parsed_hash {//Confirm that phc is valid format
            Ok(ph) => {
                match Pbkdf2.verify_password(password, &ph) {
                    Ok(_) => Ok(()),//Password confirmed
                    Err(e) => return Err(e)//Password was incorrect
                }
            },
            Err(e) => Err(e)
        }
    }

    //phc = Shorthand for a string format developed at the Password Hacking Competition. It specifies how to determine what was used to generate the hash.
    // Hash data $<algorithm>$<salt>$<hash>
    // $pbkdf2-sha256$i=10000,l=32$VI1K2hsmvxq1Urky3DQY/w$gE/SJ29k8+Esw5fJdxUrJHewh+hUkKhYhlMJjRzFjDQ  <--This is an example PHC
    // algo = pbkdf2-sha256
    // Params = i=10000,l=32 (iterations and word size?)
    // salt = VI1K2hsmvxq1Urky3DQY/w
    // hash = gE/SJ29k8+Esw5fJdxUrJHewh+hUkKhYhlMJjRzFjDQ
    //The phc is safe to store in the db because it is just the algorithm used, the salt, and the hash.
    pub fn get_phc(user_password: String) -> Result<String, pbkdf2::password_hash::errors::Error>  {// -> Result<PasswordHash<'static>, pbkdf2::password_hash::errors::Error> {
        // Hash is a one way operation: PW -> Hash algorithm = Hashed value. The user's pw is never stored.
        // Salting a PW defeats rainbow tables because it gives each input to the hash a unique value.
        // (PW + Salt) -> Algorithm = Salted and hashed value.
        // OK to store salt in plain text next to hashed value.
        // Salt should be random and unique to each user (to prevent collisions on users using the same pw)
        let password = user_password.as_bytes(); //User supplied
        let salt = SaltString::generate(&mut OsRng); //System generated salt
        match Pbkdf2.hash_password(password, &salt) {
            Ok(phc) => Ok(phc.to_string()), //A PHC. It contains meta data on the encryption method, the salt, and the generated hash.
            Err(e) => Err(e)
        }
    }

/*
    pub fn hash<'s>(
        password: &[u8],
        salt: &'s SaltString,
    ) -> Result<PasswordHash<'s>, pbkdf2::password_hash::errors::Error> {
        let password_hash = Pbkdf2.hash_password(password, salt);
        password_hash
    }
    
    fn main() {
        let password = b"password";
        let salt = SaltString::generate(&mut OsRng);
        hash(password, &salt);
    }
*/
