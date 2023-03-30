use crate::config::DbConn;
use crate::schema::{tag, blog_tags};
use crate::models::{Tag, MyResponse};
use diesel::prelude::*;
use rocket::serde::json::{Json, Value, json};
use rocket::http::{Status};
use rocket::response::status;


pub mod routes {
    use diesel::mysql::Mysql;
    use crate::models::{ErrorResponse};
    use super::*;

    #[derive(Debug, FromForm, Clone)]
    pub struct QParams {
        start: Option<i64>,
        step: Option<i64>,
        filter: Filters,
        order: Option<Vec<String>> 
        //grouped_by: Use this to handle the data from many to 1 table relations
    }

    #[derive(Debug, FromForm, Clone)]
    pub struct Filters {
        like: Option<Vec<String>>,
        eq: Option<Vec<String>>,
        ge: Option<Vec<String>>,
        le: Option<Vec<String>>,
        between: Option<Vec<String>>,
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

    pub async fn parse_and_query(params: QParams, conn: DbConn){
        //https://docs.diesel.rs/2.0.x/diesel/prelude/trait.QueryDsl.html#method.filter
        match conn.run(move |c|  {

            let mut query = tag::table.into_boxed::<Mysql>();
            if let Some(eq) = params.filter.eq {
                for f in eq {
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

        }).await {
            Ok(tags) => println!("Here are the results: {:?}", tags ),
            Err(_) => println!("Zero results"),
        }
    }

    #[get("/?<params..>")]
    pub async fn get_tags_test(params: QParams, conn: DbConn) {
        parse_and_query(params, conn).await;
    }

    #[get("/?<start>&<step>")]
    pub async fn get_tags(start: u8, step: u8, conn: DbConn) -> Value {
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
    
    #[post("/", format = "json", data="<new_tags>")]
    pub async fn add_tag(conn: DbConn, new_tags: Json<Vec<String>>) -> Result< Value, status::Custom<Value>> {
        //let tags = vec![new_tag.name.clone()];
        match helper::add_tag(&conn, new_tags.into_inner()).await 
        {
            Ok(count) => Ok(json!(count)),
            Err(e) => Err(status::Custom(Status::InternalServerError, json!(format!("Could not add entry to tag. {}", e)))),
        }
    }
}

pub mod helper {
    use super::*;
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