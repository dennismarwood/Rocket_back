use super::schema::{blog, tag, blog_tags, user, role};
use rocket::response;
use rocket::serde::json::{Value};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PayloadResponse {
    pub payload: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct InformationalResponse {
    pub message: String,
}
/*
    
    {
        "status": "success",
        "data": [
            {
                "id": 1,
                "name": "Item 1",
                "description": "This is an example item"
            }
        ]
    }

    {
        "status": "error",
        "code": 400,
        "message": "Invalid request data"
    }

 */

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AResponse {
    pub status: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<rocket::serde::json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<rocket::serde::json::Value>,
}

impl AResponse {
    pub fn success_200(data: Option<Value>) -> Self {
        AResponse { 
            status: String::from("Success"), 
            data: data,
            message: None,
            location: None,
            code: None,
            errors: None,
        }
    }
    pub fn error(message: Option<String>, code: Option<String>, 
        errors: Option<Value>) -> Self {
            AResponse {
                status: String::from("Error"),
                data: None,
                message: message,
                location: None,
                code: code,
                errors: errors,
            }
    }
}


#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MyResponse {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub payloads: Vec<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub information:Vec<InformationalResponse>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorResponse>,
}

impl MyResponse {
    pub fn new() -> Self {
        MyResponse {errors: vec![], information: vec![], payloads: vec![]}
    }
}

#[derive(serde::Serialize, Queryable, Identifiable, Debug, serde::Deserialize)]
#[table_name = "blog"]
pub struct BlogEntry {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub created: Option<chrono::NaiveDateTime>,
    pub last_updated: Option<chrono::NaiveDate>,
    pub content: Option<String>,
}

#[derive(serde::Serialize, Queryable, Identifiable, Debug)]
#[table_name="tag"]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(serde::Serialize, Queryable, Associations, Identifiable, Debug, serde::Deserialize)]
#[table_name = "blog_tags"]
#[belongs_to(BlogEntry, foreign_key = "blog_id")]
#[belongs_to(Tag)]
pub struct BlogTags {
    pub id: i32,
    pub blog_id: i32,
    pub tag_id: i32
}

#[derive(Insertable, serde::Deserialize)]
#[table_name="blog_tags"]
pub struct NewBlogTag {
    pub blog_id: i32,
    pub tag_id: i32
}

#[derive(serde::Serialize, serde::Deserialize, Identifiable, Queryable, Associations, PartialEq, Debug)]
#[table_name="user"]
#[belongs_to(Role, foreign_key = "role")]
pub struct User {
    pub id: i32,
    pub email: Option<String>,
    pub phc: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created: Option<chrono::NaiveDateTime>,
    pub role: i32,
    pub active: Option<bool>,
    pub last_access: Option<chrono::NaiveDate>,
}

#[derive(Insertable, serde::Deserialize, Queryable, Debug)]
#[table_name="user"]
pub struct NewUser {
    pub email: String,
    pub phc: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: i32,
    pub active: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Identifiable, Queryable, PartialEq, Debug)]
#[table_name="role"]
pub struct Role {
    pub id: i32,
    pub user_role: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JWTClaims {
    pub user_id: i32,
    pub email: String,
    pub role_id: i32,
    pub role: String,
    pub exp: usize
}

use rocket::serde::Deserialize;
#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct EnvVariables {
    pub jwt_secret: String,
}