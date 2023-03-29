use crate::config::DbConn;
use crate::schema::{tag, blog_tags};
use crate::models::{Tag, MyResponse};
use diesel::prelude::*;
use rocket::serde::json::{Json, Value, json};
use rocket::http::{Status};
use rocket::response::status;
use std::collections::HashMap;


pub mod routes {
    //use crate::auth::auth::Level1;

    //use diesel::result::Error;

    //use core::num::dec2flt::parse;

    use std::iter;

    use diesel::mysql::Mysql;

    use crate::models::{ErrorResponse};

    use super::*;

    //    http://localhost:8000  /tags?start=0&step=10&filters[]=id=5&filters[]=name=Rust
    //http://localhost:8001/api/tags/test/?start=0&step=10&dbfilter[filters][]=id=5&dbfilter[filters][]=name=Rust
    #[derive(Debug, FromForm, Clone)]
    pub struct Filter {
        filters: Option<Vec<String>>,
    }
    
    #[derive(Debug, FromForm, Clone)]
    pub struct QParams {
        start: Option<i64>,
        step: Option<i64>,
        filter: Option<Vec<String>>, //filter=author=Dennis
        order: Option<Vec<String>> 
        //grouped_by: Use this to handle the data from many to 1 table relations
    }

    pub enum Tables {
        Tag,
    }
    
    //pub async fn parse_and_query<T>(table: T, params: QParams, conn: DbConn)
    //where
        //T: Table,
    //{
    pub async fn parse_and_query(table: Tables, params: QParams, conn: DbConn){
        match conn.run(move |c|  {

            let mut query = match table {
                Tables::Tag => tag::table.into_boxed::<Mysql>(),
            }; 
            //let mut query = table.into_boxed::<Mysql>();
            //READ UP ON https://docs.diesel.rs/1.4.x/diesel/expression/trait.BoxableExpression.html#examples
            //filters  https://docs.diesel.rs/2.0.x/diesel/prelude/trait.QueryDsl.html#method.filter
            //http://localhost:8001/api/tags/test/?start=0&step=10&order=-id&filter=id=79&filter=id=78
            //like, between, eq, eq_any, ge, le
            //filter=name=eq=Rust
            //filter=name=like=Ru%
            //filter[like]=name=Ru%
            //filter[eq]=name=Rust
            //Maybe break down the filters to calls of functions that build the query

            if let Some(p_f) = params.filter {
                for f in p_f {
                    //f looks like "id=47"
                    if let Some((k, v)) = f.split_once('=').map(|(k, v)| (k.to_owned(), v.to_owned())) {
                        match table {
                            Tables::Tag => {
                                match k.as_str() {
                                    "id" => query = query.or_filter(tag::id.eq(v.parse::<i32>().unwrap())),
                                    "name" => query = query.or_filter(tag::name.eq(v)),
                                    _ => {},
                                }
                            }
                        }
                    }
                }
            }

            //order https://docs.diesel.rs/1.4.x/diesel/query_dsl/trait.QueryDsl.html#method.then_order_by
            //asc, desc,
            if let Some(p_o) = params.order {
                for o in p_o {
                    match table {
                        Tables::Tag => {
                            match o.as_str() {
                                "id" => query = query.then_order_by(tag::id.asc()),
                                "-id" => query = query.then_order_by(tag::id.desc()),
                                "name" => query = query.then_order_by(tag::name.asc()),
                                "-name" => query = query.then_order_by(tag::name.desc()),
                                _ => {},
                            }
                        }
                    }
                }
            }

            //page indexing
            let start: i64 = params.start.unwrap_or(0);
            let step: i64 = params.step.unwrap_or(10);
            query = query.limit(step);
            query = query.offset(start);
            match table {
                Tables::Tag => {
                    query.load::<Tag>(c)
                }
            }
            //query.select((tag::name, tag::id)).load::<(String, i32)>(c)

        }).await {
            Ok(tags) => println!("Here are the results: {:?}", tags ),
            Err(_) => println!("Zero results"),
        }
    }

    #[get("/?<params..>")]
    pub async fn get_tags_test(params: QParams, conn: DbConn) {
        //http://localhost:8001/api/tags/test/?start=0&step=10&dbfilter[filters][]=id=5&dbfilter[filters][]=name=Rust&myfilter=test=3&myfilter=test2=1&filter=id=46

        parse_and_query(Tables::Tag, params, conn).await;
        //parse_and_query(tag::table, params, conn).await;
        let x = tag::table;
        //match conn.run(move |c|  {
            //parse_and_query(params, conn);
            /*
            tag::table
            .or_filter(tag::id.eq(46))
            .select(tag::name)
            .first::<String>(c)
            */
        /* }).await {
            Ok(tags) => println!("{}", tags ),
            Err(_) => println!("Zero results"),
        }*/
    }
        /*
        parse_and_query(params.clone(), conn).await;
        let r = "get_tags_test";
        let start = params.start.unwrap_or(0) as i64;
        let step = params.step.unwrap_or(10) as i64;
        println!("dbfilter values = {:?}", &params.dbfilter);

        let mut filters_map = HashMap::new();
        if let Some(filter) = params.dbfilter {
            if let Some(filters) = filter.filters {
                for filter_pair in filters {
                    let filter_parts: Vec<&str> = filter_pair.splitn(2, '=').collect();
                    if filter_parts.len() == 2 {
                        let column = filter_parts[0].trim();
                        let value = filter_parts[1].trim();
                        println!("{}", column.to_string());
                        println!("{}", value.to_string());

                        filters_map.insert(column.to_owned(), value.to_owned());
                    }
                }
            }
        }
        */
        //let mut my_filters_map = HashMap::new()

        //format!("{}\nstart:{}\nstep:{}\n{:?}\n{:?}", r, start, step, filters_map, params.myfilter)
    //}

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