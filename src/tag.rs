use crate::config::DbConn;
use crate::schema::{tag, post_tags};
use crate::models::{Tag, AResponse, QParams, Filters, BlogTags};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind::{UniqueViolation, NotNullViolation };
use diesel::result::Error::{DatabaseError, QueryBuilderError};
use rocket::http::{Status};
use rocket::response::status;
use rocket::serde::json::{Json, json};


pub mod routes {
    use diesel::mysql::Mysql;
    use super::*;

    enum TagFields {
        Id(i32),
        Name(String),
    }
    
    #[derive(Debug, Insertable, serde::Deserialize)]
    #[diesel(table_name = tag)]
    pub struct NewTag {
        pub name: String,
    }
    
    #[derive(Debug, AsChangeset, serde::Serialize, serde::Deserialize, Identifiable)]
    #[diesel(table_name = tag)]
    struct UpdateTag {
        pub id: i32,
        pub name: Option<String>,
    }

    fn validation(qp: String) -> Option<TagFields> {
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

    pub async fn parse_and_query(params: QParams, conn: &DbConn) -> QueryResult<Vec<Tag>> {
        //https://docs.diesel.rs/2.0.x/diesel/prelude/trait.QueryDsl.html#method.filter
        conn.run(move |c| {

            let mut query = tag::table.into_boxed::<Mysql>();
            //page indexing
            let start: i64 = params.start.unwrap_or(0);
            let step: i64 = params.step.unwrap_or(QParams::calculate_limit(&params));
            query = query.offset(start);
            query = query.limit(step);

            for f in params.filter.eq {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        TagFields::Id(id) => query = query.or_filter(tag::id.eq(id)),
                        TagFields::Name(name) => query = query.or_filter(tag::name.eq(name)),
                    }
                }
            }

            for f in params.filter.ge {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        TagFields::Id(id) => query = query.or_filter(tag::id.ge(id)),
                        TagFields::Name(name) => query = query.or_filter(tag::name.ge(name)),
                    }
                }
            }

            for f in params.filter.le {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        TagFields::Id(id) => query = query.or_filter(tag::id.le(id)),
                        TagFields::Name(name) => query = query.or_filter(tag::name.le(name)),
                    }
                }
            }

            for f in &params.filter.like {
                if let Some((k, v)) = f.split_once('=') {
                    match k.to_lowercase().as_str() {
                        "name" => query = query.or_filter(tag::name.like(v)),
                        _ => {},
                    }
                }
            }

            for b in &params.filter.between {
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

            for o in params.order {
                match o.as_str() {
                    "id" => query = query.then_order_by(tag::id.asc()),
                    "-id" => query = query.then_order_by(tag::id.desc()),
                    "name" => query = query.then_order_by(tag::name.asc()),
                    "-name" => query = query.then_order_by(tag::name.desc()),
                    _ => {},
                }
            }
            
            query.load::<Tag>(c)
            

        }).await 
    }

    async fn retrieve_one_tag(post_id: i32, conn: &DbConn) -> Result< Vec<Tag>, status::Custom<Json<AResponse>> > {
        //Given an id, return a vec of Tag with a single entry or an error 404 / 500
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", post_id)]));
  
        match parse_and_query(q_params, &conn).await {
            Ok(tags) => {
                match tags.len() {
                    1 => Ok(tags),
                    0 => Err(status::Custom(Status::NotFound, Json(AResponse::_404(
                            Some(String::from("Could not locate tag with provided id.")))))),
                    _ => Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))),
                }
            },
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[get("/<id>")]
    pub async fn get(id: i32, conn: DbConn) -> Result<Json<AResponse>, status::Custom<Json<AResponse>>> {
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", id)]));
        match parse_and_query(q_params, &conn).await {
            Ok(tags) => match tags.len() {
                0 => Err(status::Custom(Status::NotFound, Json(AResponse::_404(None)))),
                _ => Ok(Json(AResponse::_200(Some(json!(tags))))),
            }    
            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[get("/?<params..>")]
    pub async fn get_tags(params: QParams, conn: DbConn) -> Result<Json<AResponse>, status::Custom<Json<AResponse>>> {
        match parse_and_query(params, &conn).await {
            Ok(tags) => 
                Ok(Json(AResponse::_200(Some(json!(tags))))),
            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[post("/", format="json", data="<new_tag>")]
    pub async fn post(conn: DbConn, new_tag: Json<NewTag>) -> Result<status::Created<String>, status::Custom<Json<AResponse>> > {
        //Diesel does not have an error code for invalid input. Manually check.
        if !(1..=100).contains(&new_tag.name.len()) {
            return Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                Some(String::from("Correct input and try again.")),
                Some(String::from("INVALID_INPUT")),
                Some(json!([{"field": "name", "message":  "Valid length is 1 to 100 chars."}]))))));
        }

        let tag_name = &new_tag.name.clone();
        
        match conn.run(move |c| {
            diesel::insert_into(tag::table)
            .values(&new_tag.into_inner())
            .execute(c)
        }).await {
            Ok(_) => {
                //Successfully created tag, now retrieve it's id
                match helper::get_a_tag_id(&conn, tag_name).await {
                    Some(id) => {
                        let uri = uri!("/api/tags/", get(id)).to_string();
                        let body = json!(AResponse::_201(Some(uri.clone()))).to_string();
                        Ok(status::Created::new(uri).body(body))
                    },
                    None => Ok(status::Created::new("")),//Or maybe this should be a 500?
                }
            },
            Err(DatabaseError(UniqueViolation, d)) => 
                Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                    Some(String::from(d.message())), 
                    Some(String::from("UNIQUE_VIOLATION")),
                    None)))),
            
            Err(DatabaseError(NotNullViolation, d)) => 
                Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                    Some(String::from(d.message())), 
                    Some(String::from("NOT_NULL_VIOLATION")),
                    None)))),
            
            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[patch("/<id>",  format="json", data="<new_tag>")]//Patch 204 400 404 422
    pub async fn patch(id: i32, conn: DbConn, new_tag: Json<NewTag>) -> Result<status::NoContent, status::Custom<Json<AResponse>>> {
        //Diesel does not have an error code for invalid input. Manually check.
        if !(1..=100).contains(&new_tag.name.len()) {
            return Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                Some(String::from("Correct input and try again.")),
                Some(String::from("INVALID_INPUT")),
                Some(json!([{"field": "name", "message":  "Valid length is 1 to 100 chars."}]))))));
        }

        match conn.run(move |c| {
            let updated_tag = UpdateTag {id, name: Some(new_tag.name.clone())};
            diesel::update(&updated_tag).set(&updated_tag).execute(c)
            
            //println!("\n{}\n", diesel::debug_query::<Mysql , _>(&x));
            //https://docs.diesel.rs/master/diesel/result/enum.Error.html
        }).await {
            Ok(rows) => {
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
                    Some(String::from("UNIQUE_VIOLATION")),
                    None)))),
            
            Err(DatabaseError(NotNullViolation, d)) => 
                Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
                    Some(String::from(d.message())), 
                    Some(String::from("NOT_NULL_VIOLATION")),
                    None)))),
            
            Err(QueryBuilderError(_)) => 
                Err(status::Custom(Status::BadRequest, Json(AResponse::_400(
                    Some(String::from("You need to provide the 'name' in the body of your request.")))))),

            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[delete("/<id>")]//Delete 204 400 404 422
    pub async fn delete(id: i32, conn: DbConn) -> Result< Json<AResponse>, status::Custom<Json<AResponse>> > {
        //Retrieve the target tag
        let target_tag = retrieve_one_tag(id, &conn).await?;

        let mut d = json!(
            {
                "name": target_tag[0].name, 
                "id": target_tag[0].id, 
                "was on blogs": 0
            }
        );

        //Remove the associated blog_tags for the tag
        let blog_tags_count = match crate::blog_tags::delete_entries(&conn, crate::blog_tags::BelongsTo::Tag(target_tag)).await {
            Ok(ok) => ok,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };

        //Now remove the tag
        match conn.run(move |c|{
            diesel::delete(tag::table.filter(tag::id.eq(id))).execute(c)
        }).await {
            Ok(_) => {
                d["Affected posts"] = json!(blog_tags_count);
                return Ok(Json(AResponse::_200(Some(d))));
            },
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[get("/<id>/posts")]
    pub async fn get_posts(id: i32, conn: DbConn) -> Result<Json<AResponse>, status::Custom<Json<AResponse>>> {
        //Retrieve the target tag
        let target_tag = retrieve_one_tag(id, &conn).await?;

        //Retrieve the post ids that have the specified tag
        match conn.run(move |c|{
            let post_ids = match BlogTags::belonging_to(&target_tag).select(post_tags::post_id).distinct().load::<i32>(c) {
                Ok(post_ids) => post_ids,
                Err(e) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
            };
            let q = post_ids.into_iter().map(|id| format!("id={id}")).collect::<Vec<String>>();
            let q_params = QParams::new_filter(Filters::new_eq(q));
            Ok(q_params)
        }).await {
            Ok(q_params) => match crate::post::routes::post_and_tags(q_params, &conn).await
            {
                Ok(posts) =>  Ok(Json(AResponse::_200(Some(json!(posts))))),                
                Err(e) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
            },
            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
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

}