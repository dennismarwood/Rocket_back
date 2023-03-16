use std::collections::HashMap;

//https://jsonapi.org/format/
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JSONAPIError {
    pub status: String, //"401"
    pub canonical: String, //Unauthorized
    pub title: String, //JWT not authorized.
    pub detail: String, //Your session is expired.
}

enum relationship {

}
//Kind of works.. but one issue is that the Vecs require square brackets even when only one entry is passed.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JSONAPIPatch {
    pub id: String, //1
    pub type_: String, //"articles"
    pub attributes: Option<HashMap<String, String>>,
    pub relationships: Option<
            Vec<
                HashMap<String,//"author"
                    HashMap<String,//"data"
                        Vec<
                            HashMap<String, String>//"type": "people"
                        >
                    >
                >
            >
    >, 
}

//To return an array of 1 or to not... https://github.com/json-api/json-api/issues/268
/*
https://serde.rs/string-or-struct.html
[12:54 AM]dennis: Hi. I am trying to load json into a struct with serde. My problem is that the json could contain an array, or it could contain just a singe entry. My struct has to be hard coded for one or the other. https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=1643981e5bdc2f9305e21da783e24d36
Rust Playground
A browser interface to the Rust compiler to experiment with the language
[12:54 AM]Piepmatz: https://serde.rs/string-or-struct.html maybe that helps you
Either string or struct · Serde
@wusticality
basically i want to iterate over one hashmap and stick things in another (types aren't the same)
[12:55 AM]Parxevicj: you should also be able to use HashMap::extend for that
[12:55 AM]Parxevicj: flowfields.extend(settings.iter().map(/*whatever*/)) (edited)
@Piepmatz
https://serde.rs/string-or-struct.html maybe that helps you
[12:58 AM]dennis: yes, I believe it might. So I will need to impl those functions to tell serde what to do depending on what arrives?
@wusticality
Do enums not implement the Copy trait?
[12:59 AM]pie_flavor: As a matter of fundamental principle, there is no overridable default behavior (except for trait methods as a concession to practicality). If there is a good default, you must invoke it explicitly. (edited)
[12:59 AM]pie_flavor: perhaps it is weird for a Lisp man, but it would not be for, for example, a Haskell man; it is from there that the word 'derive' comes, with approximately the same system involved.
@dennis
yes, I believe it might. So I will need to impl those functions to tell serde what to do depending on what arrives?
[1:00 AM]Piepmatz: I think in your struct you should have an enum that can hold either the array, so a vector in rust or the single entry
[1:00 AM]Piepmatz: or make that both become a vector (edited)
[1:01 AM]pie_flavor: a user-defined type does not automatically implement anything except the auto traits. If you want it to implement Copy, it is as simple as derive(Copy).
@Piepmatz
I think in your struct you should have an enum that can hold either the array, so a vector in rust or the single entry
[1:02 AM]dennis: OK thank you 
NEW
[1:04 AM]Piepmatz: I'm not sure if serde can automatically fill a single value into a vector if you define the field to be a vector. You maybe have to define a deserialize function for that yourself.

And if you want to use an enum for that you may need to check out tagging. Sometimes the json indicates somewhere if the value is either one or the other. If not you can try if untagged works for you.
*/