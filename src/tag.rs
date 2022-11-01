use crate::config::DbConn;
use crate::schema::{tag, blog_tags};
use crate::models::{Tag, MyResponse};
use diesel::prelude::*;
use rocket::serde::json::{Json, Value, json};
use rocket::http::{Status};
use rocket::response::status;

pub mod routes {
    //use crate::auth::auth::Level1;

    //use diesel::result::Error;

    use crate::models::{ErrorResponse};

    use super::*;

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
                response.payloads.push(json!(tags)), 

            Err(e) => 
                //status::Custom(Status::InternalServerError , json!(format!("{}", e)))
                response.errors.push(ErrorResponse { code: Status::InternalServerError.to_string(), message: e.to_string() }),
        }
        //response.information.push(InformationalResponse { message: "Here is some info".to_string() });
        json!(&response)
    }

    #[get("/?<id>", rank = 1)]
    pub async fn get_tag_by_id(id: i32, conn: DbConn) -> Value {
        let mut response = MyResponse::new();
        match conn.run(move |c|  {
            tag::table
            .filter(tag::id.eq(id))
            .select(tag::name)
            .first::<String>(c)
        }).await {
            Ok(tags) => response.payloads.push(json!(tags)),
            Err(e) => response.errors.push(ErrorResponse { code: Status::NoContent.to_string(), message: e.to_string() }),
        }
        json!(&response)
    }

    #[get("/?<name>", rank = 2)]
    pub async fn get_tag_by_name(name: String, conn: DbConn) -> Result< Value, status::Custom<Value>> {
        match conn.run(move |c|  {
            tag::table
            .filter(tag::name.eq(name))
            .select(tag::id)
            .first::<i32>(c)
        }).await {
            Ok(tag) => return Ok(json!(tag)),
            Err(e) => return Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    }

    #[get("/?<blog_id>", rank = 3)]
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
    /*#[get("/tag?<name>")]
    pub async fn get_tags(name: String, conn: DbConn) -> Value {}
 */

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