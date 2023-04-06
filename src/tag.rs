use crate::config::DbConn;
use crate::schema::{tag, blog_tags};
use crate::models::{Tag, AResponse};
use diesel::prelude::*;
use rocket::serde::json::{Json, Value, json};
use rocket::http::{Status};
use rocket::response::status;
use diesel::result::DatabaseErrorKind::{UniqueViolation, NotNullViolation };
use diesel::result::Error::{DatabaseError, QueryBuilderError};

pub mod routes {
    use diesel::mysql::Mysql;
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

        }).await 
    }

    #[get("/<id>")]
    pub async fn get_tag(id: i32, conn: DbConn) -> Result<Json<AResponse>, status::BadRequest<Json<AResponse>>> {
        let q_params = QParams::new_filter(Filters::new_eq(Some(vec![format!("id={}", id)])));
        //let uri = uri!(get_tags( q_params ));
        match parse_and_query(q_params, conn).await {
            Ok(tags) => Ok(Json(AResponse::_200(Some(json!(tags))))),
            Err(_) => Err(status::BadRequest(Some(
                Json(AResponse::_400(
                    Some(String::from("Could not perform query with given parameters. Check input before trying again."))))
                ))),
        }
    }

    #[get("/?<params..>")]
    pub async fn get_tags(params: QParams, conn: DbConn) -> Result<Json<AResponse>, status::BadRequest<Json<AResponse>>> {
        match parse_and_query(params, conn).await {
            Ok(tags) => Ok(Json(AResponse::_200(Some(json!(tags))))),
            Err(_) => Err(status::BadRequest(Some(
                Json(AResponse::_400(
                    Some(String::from("Could not perform query with given parameters. Check input before trying again."))))
                ))),
        }
    }


    //A post should return 201 with an address to the new entry in the locaiton header and some additional data in the body.
    //The post may fail for several reasons. All fails have a default response by rocket.
    //For the case of a 403 Forbidden (user is unauthorized) then I may want to write a custom handler. See rocket doc under responses. 
    //Cases that should be handled because specific error messages should be presented to user are:
    //400 (Bad Request). Default handler is currently being called for fails to match on NewTag.
    //422 (Unprocessable Entity) Invalid input (that still meets the criteria of the data guard) or a duplicate entry.
    //TODO: The default handlers will be firing on guard fails, better to replace those with AResponse types.

    #[post("/", format="json", data="<new_tag>")]
    pub async fn post_tag(conn: DbConn, new_tag: Json<NewTag>) -> Result<status::Created<String>, status::Custom<Json<AResponse>> > {
        //Because we are loading into a struct, rocket will guard against missing key value in name: string. returns 422.
        //So there is no need to check that the key is present, only that it is valid.
        /* if !(1..=100).contains(&new_tag.name.len()) {
            return Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                Some(String::from("Correct input and try again.")), 
                Some(String::from("422")), 
                Some(json!([{"field": "name", "message":  "Valid length is 1 to 100 chars."}]))))));
        } */

        let tag_name = &new_tag.name.clone();
        
        match conn.run(move |c| {
            diesel::insert_into(tag::table)
            .values(tag::name.eq(&new_tag.name))
            .execute(c)
        }).await {
            Ok(_) => {
                //Successfully created tag, now retrieve it's id
                match helper::get_a_tag_id(&conn, tag_name).await {
                    Some(id) => {
                        let uri = uri!("/api/tags/", get_tag(id)).to_string();
                        let body = json!(AResponse::_201(Some(uri.clone()))).to_string();
                        Ok(status::Created::new(uri).body(body))
                    },
                    None => Ok(status::Created::new("")),
                }
            },
            Err(DatabaseError(UniqueViolation, d)) => 
                Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                    Some(String::from(d.message())), 
                    Some(String::from("UNIQUE_VIOLATION")))))),
            
            Err(DatabaseError(NotNullViolation, d)) => 
                Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                    Some(String::from(d.message())), 
                    Some(String::from("NOT_NULL_VIOLATION")))))),
            
            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[patch("/<id>",  format="json", data="<updated_tag>")]
    pub async fn patch_tag(id: i32, conn: DbConn, mut updated_tag: Json<UpdateTag>) -> Result<status::NoContent, status::Custom<Json<AResponse>> > {
        match conn.run(move |c| {
            /*
            (Newer user here, so assume it is something obvious)
I am having an issue performing a record update when trying to use the "Identifiable" trait and the "AsChangeSet" trait.
Things do work if I alter the query to include a filter on the id.
https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=d24d8eadbd6efce2a31a605b4484a91a
Why doesn't it include the id as I would expect? Using version 1.4.1
             */
            updated_tag.id = Some(id);
            //println!("{:?}", updated_tag);
            //diesel is able to deduce the row based on the id specified in the updated_tag.
            //The filtering occurs automatically from the "Identifiable" macro on the Tag model.
            let x = //diesel::update(tag::table)
            diesel::update(tag::table.filter(tag::id.eq(id)))
            .set(updated_tag.into_inner())
            ;//.execute(c)
            
            println!("\n{}\n", diesel::debug_query::<Mysql , _>(&x));
            x.execute(c)
        }).await {
            //NO: DatabaseError(UniqueViolation, "Duplicate entry 'Patched' for key 'tag.UC_name'")
            //https://docs.diesel.rs/master/diesel/result/enum.Error.html
            Ok(rows) => {
                println!("{}", rows); 
                match rows {
                    1 => Ok(status::NoContent),
                    0 => Err(status::Custom(Status::NotFound, Json(AResponse::_404(
                            Some(String::from("Could not locate tag with provided id.")))))),
                    _ => Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))),
                }
            }
            Err(DatabaseError(UniqueViolation, d)) => 
                Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                    Some(String::from(d.message())), 
                    Some(String::from("UNIQUE_VIOLATION")))))),
            
            Err(DatabaseError(NotNullViolation, d)) => 
                Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                    Some(String::from(d.message())), 
                    Some(String::from("NOT_NULL_VIOLATION")))))),
            
            Err(QueryBuilderError(_)) => 
                Err(status::Custom(Status::BadRequest, Json(AResponse::_400(
                    Some(String::from("You need to provide the 'name' in the body of your request.")))))),

            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[derive(Debug, Insertable, serde::Deserialize)]
    #[table_name="tag"]
    pub struct NewTag {
        pub name: String,
    }
    
    #[derive(Debug, AsChangeset, serde::Serialize, serde::Deserialize, Identifiable)]//Identifiable
    #[table_name="tag"]
    pub struct UpdateTag {
        pub id: Option<i32>,
        pub name: Option<String>,
    }

}

pub mod helper {
    use super::*;
    //mysql does not return an id after creating a new entry. This helper function does only that.
    pub async fn get_a_tag_id( conn: &DbConn, name: &String) -> Option<i32> {
        let name = name.clone();
        match conn.run(  |c| {
            tag::table
                .filter(tag::name.eq(name))
                .select(tag::id)
                .first::<i32>(c)
        }).await {
            Ok(id) => Some(id),
            Err(_) => None,
        }
    }

    pub async fn get_tag_ids(conn: &DbConn, name: Vec<String>) -> Result <Vec<i32>, diesel::result::Error> {
        conn.run(  |c| {
            tag::table
                .filter(tag::name.eq_any(name))
                .select(tag::id)
                .load::<i32>(c)
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
}