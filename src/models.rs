use super::schema::{blog, tag, blog_tags, user, role};
use rocket::serde::json::{Value};

#[derive(Debug, FromForm)]
    pub struct QParams {
        pub start: Option<i64>,
        pub step: Option<i64>,
        pub filter: Filters,
        pub order: Vec<String> 
        //grouped_by: Use this to handle the data from many to 1 table relations
    }
    
    impl QParams {
        pub fn new_filter(filter: Filters) -> Self {
            QParams {
                filter,
                start: None,
                step: None,
                order: Vec::new(),
            }
        }

        pub fn calculate_limit(&self) -> i64 {
            /*
            Some functions, such as the put on posts, can purposely pass empty filter values
            or the user may pass invalid values thus effectively passing empty filter values.
            Empty filters will return all entries as it is an sql select w/o any where clause.
            To address this, look at all the filters. If they are empty assume the user does NOT
            acutally want an unconstrained search and return an empty set to them. 

            This method should only be called if the user did not specify a step value in their query.
            */

            if  self.filter.like.len() +
                self.filter.eq.len() +
                self.filter.ge.len() +
                self.filter.le.len() +
                self.filter.between.len() == 0 {return 0}
            //10 is the default value if not specified by the user.
            10
        }
    }
    
    #[derive(Debug, FromForm)]
    pub struct Filters {
        pub like: Vec<String>,
        pub eq: Vec<String>,
        pub ge: Vec<String>,
        pub le: Vec<String>,
        pub between: Vec<String>,
    }
    
    impl Filters {
        pub fn new_eq(eq: Vec<String>) -> Self {
            Filters {
                eq,
                like: Vec::new(),
                ge: Vec::new(),
                le: Vec::new(),
                between: Vec::new(), 
            }
        }
    }

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
    pub fn _200(data: Option<Value>) -> Self {
        AResponse { 
            status: String::from("Success"), 
            data: data,
            message: None,
            location: None,
            code: None,
            errors: None,
        }
    }
    pub fn _201(location: Option<String>) -> Self {
        AResponse { 
            status: String::from("Success"), 
            data: None,
            message: Some(String::from("Resource created Successfully")),
            location: location,
            code: None,
            errors: None,
        }
    }
    pub fn _400(message: Option<String>) -> Self {
        AResponse {
            status: String::from("Error"),
            data: None,
            message: message,
            location: None,
            code: Some(String::from("INVALID_USER_INPUT")),
            errors: None,
        }
    }
    pub fn _404(message: Option<String>) -> Self {
        AResponse {
            status: String::from("Error"),
            data: None,
            message: message,
            location: None,
            code: Some(String::from("NOT_FOUND")),
            errors: None,
        }
    }
    pub fn _422(message: Option<String>, code: Option<String>, errors: Option<Value>) -> Self {
            AResponse {
                status: String::from("Error"),
                data: None,
                message: message,
                location: None,
                code: code,
                errors: errors,
            }
    }
    pub fn _500() -> Self {
        AResponse {
            status: String::from("Error"),
            data: None,
            message: Some(String::from("Our apologies, something went wrong.")),
            location: None,
            code: Some(String::from("INTERNAL_SERVER_ERROR")),
            errors: None,
        }
}
    pub fn error(errors: Option<Value>) -> Self {
            AResponse {
                status: String::from("Error"),
                data: None,
                message: Some(String::from("An unexpected error type has occured.")),
                location: None,
                code: Some(String::from("UNEXPECTED_ERROR_TYPE")),
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

#[derive(serde::Serialize, Queryable, Identifiable, Debug, serde::Deserialize, Clone)]
#[diesel(table_name = blog)]
pub struct BlogEntry {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub created: Option<chrono::NaiveDateTime>,
    pub last_updated: Option<chrono::NaiveDate>,
    pub content: Option<String>,
}

#[derive(serde::Serialize, Queryable, Identifiable, Debug, serde::Deserialize, AsChangeset, Selectable, PartialEq)]
#[diesel(table_name = tag)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(serde::Serialize, Queryable, Associations, Identifiable, Debug, serde::Deserialize, Selectable, Insertable)]
#[diesel(table_name = blog_tags)]
#[diesel(belongs_to(BlogEntry, foreign_key = blog_id))]
#[diesel(belongs_to(Tag))]
pub struct BlogTags {
    pub id: i32,
    pub blog_id: i32,
    pub tag_id: i32
}

#[derive(Insertable, serde::Deserialize)]
#[diesel(table_name = blog_tags)]
pub struct NewBlogTag {
    pub blog_id: i32,
    pub tag_id: i32
}

#[derive(serde::Serialize, serde::Deserialize, Identifiable, Queryable, Associations, PartialEq, Debug)]
#[diesel(table_name = user)]
#[diesel(belongs_to(Role, foreign_key = role))]
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
#[diesel(table_name = user)]
pub struct NewUser {
    pub email: String,
    pub phc: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: i32,
    pub active: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Identifiable, Queryable, PartialEq, Debug)]
#[diesel(table_name = role)]
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