


//https://jsonapi.org/format/
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JSONAPIError {
    pub status: String, //"401"
    pub canonical: String, //Unauthorized
    pub title: String, //JWT not authorized.
    pub detail: String, //Your session is expired.
}