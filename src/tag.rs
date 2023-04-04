use crate::config::DbConn;
use crate::schema::{tag, blog_tags};
use crate::models::{Tag, MyResponse, AResponse};
use diesel::prelude::*;
use rocket::serde::json::{Json, Value, json};
use rocket::http::{Status};
use rocket::response::status;
use rocket::http::uri::fmt::{Formatter, UriDisplay, FromUriParam, Query};
use std::fmt;

pub mod routes {
    use chrono::format::parse;
    use diesel::mysql::Mysql;
    use rocket::serde::json;
    use crate::models::{ErrorResponse};
    use super::*;

    #[derive(Debug, FromForm, Clone, UriDisplayQuery)]
    pub struct QParams {
        start: Option<i64>,
        step: Option<i64>,
        filter: Filters,
        order: Option<Vec<String>> 
        //grouped_by: Use this to handle the data from many to 1 table relations
    }
    
    impl QParams {
        pub fn new_filter(filter: Filters) -> Self {
            QParams {
                filter,
                start: None,
                step: None,
                order: None,
            }
        }
    }

 /* impl UriDisplay<Query> for QParams {
        fn fmt(&self, f: &mut Formatter<Query>) -> fmt::Result {
            if let Some(start) = self.start {
                f.write_named_value("start", &start)?;
            }
            if let Some(step) = self.step {
                f.write_named_value("step", &step)?;
            }
            f.write_named_value("filter", &self.filter)?;
            if let Some(order) = &self.order {
                for (i, value) in order.iter().enumerate() {
                    if i > 0 {
                        f.write_value("&")?;
                    }
                    f.write_named_value("order[]", value)?;
                }
            }
            Ok(())
        }
    }

    impl FromUriParam<Query, (Option<i64>, Option<i64>, Filters, Option<Vec<String>>)> for QParams {
        type Target = QParams;

        fn from_uri_param(param: (Option<i64>, Option<i64>, Filters, Option<Vec<String>>)) -> QParams {    
            let (start, step, filter, order) = param;
            QParams {
                start,
                step,
                filter,
                order,
            }
    } */
//}
    
    #[derive(Debug, FromForm, Clone, UriDisplayQuery)]
    pub struct Filters {
        like: Option<Vec<String>>,
        eq: Option<Vec<String>>,
        ge: Option<Vec<String>>,
        le: Option<Vec<String>>,
        between: Option<Vec<String>>,
    }
    
    impl Filters {
        pub fn new_eq(eq: Option<Vec<String>>) -> Self {
            Filters {
                eq,
                like: None,
                ge: None,
                le: None,
                between: None, 
            }
        }
    }
    pub enum TagFields {
        Id(i32),
        Name(String),
    }

    pub fn validation(qp: String) -> Option<TagFields> {
        // Verify that the query parameter is valid in format
        if let Some((k, v)) = qp.split_once('=') {
            match k.to_lowercase().as_str() {
                "id" => {
                    match v.parse::<i32>() {
                        Ok(v) => Some(TagFields::Id(v)),
                        _ => None,
                    }
                },
                "name" => Some(TagFields::Name(String::from(v))),
                _ => None,
            }
        } else {None}
    }

    pub async fn parse_and_query(params: QParams, conn: DbConn) -> QueryResult<Vec<Tag>> {
        //https://docs.diesel.rs/2.0.x/diesel/prelude/trait.QueryDsl.html#method.filter
        conn.run(move |c| {

            let mut query = tag::table.into_boxed::<Mysql>();
            println!("{:?}",params);
            if let Some(eq) = params.filter.eq {
                for f in eq {
                    println!("ASDF");
                    if let Some(query_parameter) = validation(f){
                        match query_parameter {
                            TagFields::Id(id) => query = query.or_filter(tag::id.eq(id)),
                            TagFields::Name(name) => query = query.or_filter(tag::name.eq(name)),
                        }
                    }
                }
            }

            if let Some(ge) = params.filter.ge {
                for f in ge {
                    if let Some(query_parameter) = validation(f){
                        match query_parameter {
                            TagFields::Id(id) => query = query.or_filter(tag::id.ge(id)),
                            TagFields::Name(name) => query = query.or_filter(tag::name.ge(name)),
                        }
                    }
                }
            }

            if let Some(le) = params.filter.le {
                for f in le {
                    if let Some(query_parameter) = validation(f){
                        match query_parameter {
                            TagFields::Id(id) => query = query.or_filter(tag::id.le(id)),
                            TagFields::Name(name) => query = query.or_filter(tag::name.le(name)),
                        }
                    }
                }
            }

            if let Some(like) = &params.filter.like{
                for f in like {
                    if let Some((k, v)) = f.split_once('=') {
                        match k.to_lowercase().as_str() {
                            "name" => query = query.or_filter(tag::name.like(v)),
                            _ => {},
                        }
                    }
                }
            }
            
            if let Some(between) = &params.filter.between {
                for b in between {
                    if let Some((k, v)) = b.split_once('=') {
                        match k.to_lowercase().as_str() {
                            "id" => {
                                if let Some((l, r)) = v.split_once(',') {
                                    match l.parse::<i32>() {
                                        Ok(l) => {
                                            match r.parse::<i32>() {
                                                Ok(r) => query = query.or_filter(tag::id.between(l, r)),
                                                _ => {},
                                            }
                                        },
                                        _ => {},
                                    }
                                }
                            }, 
                            "name" => {
                                if let Some((l, r)) = v.split_once(',') {
                                    query = query.or_filter(tag::name.between(l, r));
                                }
                            },
                            _ => {},
                        }

                    }
                }
            }

            if let Some(p_o) = params.order {
                for o in p_o {
                    match o.as_str() {
                        "id" => query = query.then_order_by(tag::id.asc()),
                        "-id" => query = query.then_order_by(tag::id.desc()),
                        "name" => query = query.then_order_by(tag::name.asc()),
                        "-name" => query = query.then_order_by(tag::name.desc()),
                        _ => {},
                    }
                }
            }

            //page indexing
            let start: i64 = params.start.unwrap_or(0);
            let step: i64 = params.step.unwrap_or(10);
            query = query.limit(step);
            query = query.offset(start);
            query.load::<Tag>(c)

        }).await 
    }


    #[get("/?<params..>")]
    pub async fn get_tags(params: QParams, conn: DbConn) -> Result<Json<AResponse>, status::BadRequest<Json<AResponse>>> {
        println!("HERE");
        match parse_and_query(params, conn).await {
            Ok(tags) => Ok(Json(AResponse::success_200(Some(json!(tags))))),
            Err(e) => Err(status::BadRequest(Some(
                Json(AResponse::error(
                    Some(String::from("Could not perform query with given parameters. Check input before trying again.")), 
                    Some(String::from("INVALID_INPUT")), 
                    Some(json!({"errors": [{"db": e.to_string()}]}))))
                ))),
        }
    }

    #[get("/<id>")]
    pub async fn get_tag(id: i32, conn: DbConn) {//} -> Result<Json<AResponse>, status::Custom<Json<AResponse>> > {
        let x = Filters::new_eq(Some(vec![format!("id={}", id)]));
        let q_params = QParams::new_filter(x);
        //let uri = uri!(get_tags( Some(String::from("filter.eq=id=1234")) ));
        let uri = uri!(get_tags( q_params ));
        //println!("here - {}", uri.to_string());
    }

    //A post should return 201 with an address to the new entry in the locaiton header and some additional data in the body.
    //The post may fail for several reasons. All fails have a default response by rocket.
    //For the case of a 403 Forbidden (user is unauthorized) then I may want to write a custom handler. See rocket doc under responses. 
    //Cases that should be handled because specific error messages should be presented to user are:
    //400 (Bad Request). Default handler is currently being called for fails to match on NewTag.
    //422 (Unprocessable Entity) Invalid input (that still meets the criteria of the data guard) or a duplicate entry.
    //TODO: The default handlers will be firing on guard fails, better to replace those with AResponse types.

    #[post("/", format="json", data="<new_tag>")]
    pub async fn add_tag(conn: DbConn, new_tag: Json<NewTag>) -> Result<status::Created<String>, status::Custom<Json<AResponse>> > {
        //Because we are loading into a struct, rocket will guard against missing key value in name: string. returns 422.
        //So there is no need to check that the key is present, only that it is valid.
        if !(1..=100).contains(&new_tag.name.len()) {
            return Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::error(
                Some(String::from("Correct input and try again.")), 
                Some(String::from("422")), 
                Some(json!([{"field": "name", "message":  "Valid length is 1 to 100 chars."}]))))));
        }

        let tag_name = &new_tag.name.clone();
        
        match conn.run(move |c| {
            diesel::insert_into(tag::table)
            .values(tag::name.eq(&new_tag.name))
            .execute(c)
        }).await {
            Ok(_) => {
                let id = helper::get_a_tag_id(&conn, tag_name).await; //What is our newly minted tag id?
                Ok(status::Created::new(format!("Create a type-safe route uri after get /tags/{} is done", id)))    
            },
            Err(e) => Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::error(
                Some(String::from("The entry you are trying to add is already present.")), 
                Some(String::from("ENTRY_PRESENT")), 
                Some(json!([{"field": "name", "message": e.to_string()}])))))),
        }
    }

    /* #[post("/", format = "json", data="<new_tags>")]
    pub async fn add_tag(conn: DbConn, new_tags: Json<Vec<String>>) -> Result< Value, status::Custom<Value>> {
        //let tags = vec![new_tag.name.clone()];
        match helper::add_tag(&conn, new_tags.into_inner()).await 
        {
            Ok(count) => Ok(json!(count)),
            Err(e) => Err(status::Custom(Status::InternalServerError, json!(format!("Could not add entry to tag. {}", e)))),
        }
    } */

    #[get("/?<start>&<step>")]
    pub async fn get_tags_(start: u8, step: u8, conn: DbConn) -> Value {
        //let mut response = MyResponse {errors: vec![], information: vec![], payloads: vec![]};
        let mut response = MyResponse::new();
        match conn.run(move |c| {
            tag::table
            .limit(step.try_into().unwrap())
            .offset(start.try_into().unwrap())
            .load::<Tag>(c)
        }).await {
            Ok(tags) => 
                //return Ok(json!({"payload": tags}));
                match tags.len() {
                    1.. => response.payloads.push(json!(tags)),
                    _ => response.errors.push(ErrorResponse { code: Status::NoContent.to_string(), message: "The URI format was valid but the provided query parameters yielded zero results.".to_string() }),
                }
            Err(e) => 
                //status::Custom(Status::InternalServerError , json!(format!("{}", e)))
                response.errors.push(ErrorResponse { code: Status::InternalServerError.to_string(), message: e.to_string() }),
        }
        //response.information.push(InformationalResponse { message: "Here is some info".to_string() });
        json!(&response)
    }

    #[get("/<id>")]
    pub async fn get_tag_by_id(id: i32, conn: DbConn) -> Value {
        let mut response = MyResponse::new();
        match conn.run(move |c|  {
            tag::table
            .filter(tag::id.eq(id))
            .select(tag::name)
            .first::<String>(c)
        }).await {
            Ok(tags) => response.payloads.push(json!(tags)),
            Err(_) => response.errors.push(ErrorResponse { code: Status::NotFound.to_string(), message: "The specified URI does not exist because the item id was not found.".to_string() }),
        }
        json!(&response)
    }

    #[get("/?<name>")]
    pub async fn get_tag_by_name(name: String, conn: DbConn) -> Value {
        let mut response = MyResponse::new();
        match conn.run(move |c|  {
            tag::table
            .filter(tag::name.eq(name))
            .select(tag::id)
            .first::<i32>(c)
        }).await {
            Ok(tags) => response.payloads.push(json!(tags)),
            Err(_) => response.errors.push(ErrorResponse { code: Status::NotFound.to_string(), message: "The specified URI does not exist because the item name was not found.".to_string() }),
        }
        json!(&response)
    }

    #[derive(Debug, Insertable, serde::Deserialize)]
    #[table_name="tag"]
    pub struct NewTag {
        pub name: String,
    }
    
}

pub mod helper {
    use super::*;
    //mysql does not return an id after creating a new entry. This helper function does only that.
    pub async fn get_a_tag_id(conn: &DbConn, name: &String) -> String {
        let name = name.clone();
        match conn.run(  |c| {
            tag::table
                .filter(tag::name.eq(name))
                .select(tag::id)
                .first::<i32>(&*c)
        }).await {
            Ok(id) => id.to_string(),
            Err(_) => String::new(),
        }
    }

    pub async fn get_tag_ids(conn: &DbConn, name: Vec<String>) -> Result <Vec<i32>, diesel::result::Error> {
        conn.run(  |c| {
            tag::table
                .filter(tag::name.eq_any(name))
                .select(tag::id)
                .load::<i32>(&*c)
        }).await
    }

    pub async fn get_tags_on_post(blog_id: i32, conn: DbConn) -> Result< Value, status::Custom<Value>> {
        match conn.run(move |c| {
            blog_tags::table
            .inner_join(tag::table)
            .filter(blog_tags::blog_id.eq(blog_id))
            .select((tag::id, tag::name))
            .load::<Tag>(c)
            }).await
        {
            Ok(results) => return Ok(json!(results)),
            Err(e) => return Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    }
    
    pub async fn add_tag(conn: &DbConn, mut new_tags: Vec<String>) -> Result< usize, diesel::result::Error> {
        //The tag.name column has a unique constraint on it. Any duplicates passed into it with the ".values"
        //will cause all the entries to fail.

        let new_tags_copy = new_tags.clone();

        let duplicate_tags = conn.run(|c| {
            tag::table
            .filter(tag::name.eq_any(new_tags_copy))
            .select(tag::name)
            .load::<String>(c)
        }).await?;

        new_tags.retain(|x| {!duplicate_tags.contains(&x.to_string())});

        let entries: Vec<_>  = new_tags.into_iter().map(|s| tag::name.eq(s)).collect();

        conn.run(|c| {
            diesel::insert_into(tag::table)
            .values(entries)
            .execute(c)
        }).await

    }
    
    /*
    //new_tags is type `Vec<String>`
let (new_tags, tags_result) = conn.run(move |c| {
            let tags_result = tag::table
            .filter(tag::name.eq_any(new_tags))
            .select(tag::name)
            .load::<String>(c);
          (new_tags, tags_result)
        }).await;
    */
    /*
    diesel::delete(tag::table.find(id)).execute(c)
    status::NoContent
    */
    

}