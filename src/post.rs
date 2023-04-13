use rocket::serde::json::{Json, Value, json};
use crate::schema::{blog, blog_tags, tag};
use crate::models::{BlogEntry, AResponse, QParams, Filters, BlogTags, Tag};
use diesel::prelude::*;
use crate::config::DbConn;
use crate::tag::helper::{add_tag};
use crate::blog_tags::*;
use rocket::http::{Status};
use rocket::response::status;
use diesel::mysql::Mysql;

pub mod routes {
    use crate::auth::Level1;

    use super::*;
    use helper::*;
    use serde::Deserialize;

    pub enum PostFields {
        Id(i32),
        Title(String),
        Author(String),
        Created(chrono::NaiveDateTime),
        LastUpdated(chrono::NaiveDate),
        Content(String,)
    }

    pub fn validation(qp: String) -> Option<PostFields> {
        // Verify that the query parameter is valid in format
        if let Some((k, v)) = qp.split_once('=') {
            match k.to_lowercase().as_str() {
                "id" => {
                    match v.parse::<i32>() {
                        Ok(v) => Some(PostFields::Id(v)),
                        _ => None,
                    }
                },
                "title" => Some(PostFields::Title(String::from(v))),
                "author" => Some(PostFields::Author(String::from(v))),
                "content" => Some(PostFields::Content(String::from(v))),
                "created" => {
                    match chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                        Ok(d) => Some(PostFields::Created(d.and_hms(0, 0, 0))),
                        _ => None,
                    }
                }
                "lastupdated" => {
                    match chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                        Ok(d) => Some(PostFields::LastUpdated(d)),
                        _ => None,
                    }
                }
                _ => None,
            }
        } else {None}
    }

    pub async fn parse_and_query(params: QParams, conn: &DbConn) -> QueryResult<Vec<BlogEntry>> {
        //https://docs.diesel.rs/2.0.x/diesel/prelude/trait.QueryDsl.html#method.filter
        conn.run(move |c| {

            let mut query = blog::table.into_boxed::<Mysql>();

            for f in params.filter.eq {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        PostFields::Id(id) => query = query.or_filter(blog::id.eq(id)),
                        PostFields::Title(title) => query = query.or_filter(blog::title.eq(title)),
                        PostFields::Author(author) => query = query.or_filter(blog::author.eq(author)),
                        PostFields::Created(created) => query = query.or_filter(blog::created.eq(created)),
                        PostFields::LastUpdated(lu) => query = query.or_filter(blog::last_updated.eq(lu)),
                        PostFields::Content(content) => query = query.or_filter(blog::content.eq(content)),
                    }
                }
            }

            for f in params.filter.ge {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        PostFields::Id(id) => query = query.or_filter(blog::id.ge(id)),
                        PostFields::Title(title) => query = query.or_filter(blog::title.ge(title)),
                        PostFields::Author(author) => query = query.or_filter(blog::author.ge(author)),
                        PostFields::Created(created) => query = query.or_filter(blog::created.ge(created)),
                        PostFields::LastUpdated(lu) => query = query.or_filter(blog::last_updated.ge(lu)),
                        PostFields::Content(content) => query = query.or_filter(blog::content.ge(content)),
                    }
                }
            }

            for f in params.filter.le {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        PostFields::Id(id) => query = query.or_filter(blog::id.le(id)),
                        PostFields::Title(title) => query = query.or_filter(blog::title.le(title)),
                        PostFields::Author(author) => query = query.or_filter(blog::author.le(author)),
                        PostFields::Created(created) => query = query.or_filter(blog::created.le(created)),
                        PostFields::LastUpdated(lu) => query = query.or_filter(blog::last_updated.le(lu)),
                        PostFields::Content(content) => query = query.or_filter(blog::content.le(content)),
                    }
                }
            }

            for f in params.filter.like {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        PostFields::Title(title) => query = query.or_filter(blog::title.like(title)),
                        PostFields::Author(author) => query = query.or_filter(blog::author.like(author)),
                        PostFields::Content(content) => query = query.or_filter(blog::content.like(content)),
                        _ => {},
                    }
                }
            }

            for b in &params.filter.between {
                if let Some((k, v)) = b.split_once('=') {
                    if let Some((l, r)) = v.split_once(',') {
                        match k.to_lowercase().as_str() {
                            "id" => if let (Ok(l), Ok(r)) = (l.parse::<i32>(), r.parse::<i32>()) {
                                        query = query.or_filter(blog::id.between(l, r));
                                    }
                            "title" => query = query.or_filter(blog::title.between(l, r)),
                            "created" => if let (Ok(l), Ok(r)) = (
                                            chrono::NaiveDate::parse_from_str(l, "%Y-%m-%d"), 
                                            chrono::NaiveDate::parse_from_str(r, "%Y-%m-%d"),
                                        )
                                        {
                                            query = query.or_filter(blog::created.between(l.and_hms(0, 0, 0), r.and_hms(0, 0, 0)));
                                        }
                            "lastupdated" => if let (Ok(l), Ok(r)) = (
                                            chrono::NaiveDate::parse_from_str(l, "%Y-%m-%d"), 
                                            chrono::NaiveDate::parse_from_str(r, "%Y-%m-%d"),
                                        )
                                        {
                                            query = query.or_filter(blog::last_updated.between(l,r));
                                        } 
                            _ => (),
                        }
                    }
                }
            }

            for o in params.order {
                println!("HERE {}", o);
                match o.to_lowercase().as_str() {
                    "id" => query = query.then_order_by(blog::id.asc()),
                    "-id" => query = query.then_order_by(blog::id.desc()),
                    "title" => query = query.then_order_by(blog::title.asc()),
                    "-title" => query = query.then_order_by(blog::title.desc()),
                    "author" => query = query.then_order_by(blog::author.asc()),
                    "-author" => query = query.then_order_by(blog::author.desc()),
                    "created" => query = query.then_order_by(blog::created.asc()),
                    "-created" => query = query.then_order_by(blog::created.desc()),
                    "lastupdated" => query = query.then_order_by(blog::last_updated.asc()),
                    "-lastupdated" => query = query.then_order_by(blog::last_updated.desc()),
                    _ => {},
                }
            }

            //page indexing
            let start: i64 = params.start.unwrap_or(0);
            let step: i64 = params.step.unwrap_or(10);
            query = query.limit(step);
            query = query.offset(start);
            query.load::<BlogEntry>(c)

        }).await 
    }

    #[get("/?<params..>")]
    pub async fn get_posts(params: QParams, conn: DbConn) -> Result<Json<AResponse>, status::Custom<Json<AResponse>>> {
        //https://diesel.rs/guides/relations.html#many-to-many-or-mn
        //https://docs.rs/diesel/latest/diesel/prelude/trait.QueryDsl.html#method.group_by
        //http://localhost:8001/api/posts?filter.eq=id=31&filter.eq=id=33

        let target_posts = match parse_and_query(params, &conn).await {
            Ok(posts) => posts,
                //Ok(Json(AResponse::_200(Some(json!(posts))))),
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };
        
        conn.run(move |c| {

            let tags_x: Vec<(BlogTags, Tag)> = match BlogTags::belonging_to(&target_posts) 
                    .inner_join(tag::table)
                    .select(  (BlogTags::as_select(), Tag::as_select()) )
                    .load(c)
            {
                Ok(tags_x) => tags_x,
                Err(e) =>  return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
            };

            let test = tags_x
            .grouped_by(&target_posts);
            println!("{:?}", &target_posts);
            println!("{:?}", test);

            let tags: Vec<(BlogTags, Tag)> = match BlogTags::belonging_to(&target_posts) 
            .inner_join(tag::table)
            .select(  (BlogTags::as_select(), Tag::as_select()) )
            .load(c)
            {
                Ok(tags) => tags,
                Err(e) =>  return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
            };

            let posts_and_their_tags: Vec<(BlogEntry, Vec<Tag>)> = tags
                .grouped_by(&target_posts) //Vec<()
                .into_iter() //Vec<(target_poss
                .zip(target_posts) // 
                .map(|(bt, be)|
                        (be, bt.into_iter().map(|(_, bt)| bt)
                    .collect()))
                .collect();
            println!("{:?}", &posts_and_their_tags);
            println!("{}", json!(&posts_and_their_tags));
            Ok(Json(AResponse::_200(Some(json!(posts_and_their_tags)))))
        }).await

    }

    #[get("/<id>")]
    pub async fn get_post(id: i32, conn: DbConn) -> Result<Json<AResponse>, status::Custom<Json<AResponse>>> {
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", id)]));
        match parse_and_query(q_params, &conn).await {
            Ok(posts) => match posts.len() {
                0 => Err(status::Custom(Status::NotFound, Json(AResponse::_404(None)))),
                _ => Ok(Json(AResponse::_200(Some(json!(posts))))),
            }    
            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    /* #[get("/<id>")]
    pub async fn get_post_by_id(id: i32, conn: DbConn) -> Result< Value, status::Custom<Value>> {
        match conn.run(move |c| {
            blog::table
                .filter(blog::id.eq(id))
                .load::<BlogEntry>(c)
            }).await
        {
            Ok(entry) => {
                let x = get_blog_entries_and_tags(&conn, entry).await;
                return Ok(json!(x))},
            Err(e) => Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    } */

    #[get("/<id>/tags")]
    pub async fn get_tags_on_post(id: i32, conn: DbConn) -> Result< Value, status::Custom<Value>> {
        match conn.run(move |c| {
            blog::table
                .filter(blog::id.eq(id))
                .load::<BlogEntry>(c)
            }).await
        {
            Ok(entry) => {
                let x = get_blog_entries_and_tags(&conn, entry).await;
                return Ok(json!(x))},
            Err(e) => Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    }

    #[get("/title?<titles>")]
    pub async fn get_post_by_title(titles: Vec<String>, conn: DbConn) -> Result< Value, status::Custom<Value>> {
        println!("{:?}", titles);
        match conn.run(move |c| {
            blog::table
                .order(blog::id.desc())
                .filter(blog::title.eq_any(titles))
                .load::<BlogEntry>(c)
            }).await
        {
            Ok(entries) => {
                let x = get_blog_entries_and_tags(&conn, entries).await;
                return Ok(json!(x))},
            Err(e) => Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    }
 
    #[get("/author?<authors>")]
    pub async fn get_post_by_author(authors: Vec<String>, conn: DbConn) -> Result< Value, status::Custom<Value>> {
        match conn.run(move |c| {
            blog::table
                .order(blog::id.desc())
                .filter(blog::author.eq_any(authors))
                .load::<BlogEntry>(c)
            }).await
        {
            Ok(entries) => {
                let x = get_blog_entries_and_tags(&conn, entries).await;
                return Ok(json!(x))},
            Err(e) => Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    }

     #[get("/?<tags>", rank = 12)]
    pub async fn get_post_by_tags(tags: Vec<String>, conn: DbConn) -> Result< Value, status::Custom<Value>> {
        let blog_ids = match conn.run(move |c| {
            blog_tags::table
            .inner_join(tag::table)
            .filter(tag::name.eq_any(tags))
            .select(blog_tags::blog_id)
            .distinct()
            .load::<i32>(c)
            }).await
        {
            Ok(results) => results,
            Err(e) => return Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        };
    
        match conn.run(|c| {
            blog::table
            .order(blog::id.desc())
            .filter(blog::id.eq_any(blog_ids))
            .load::<BlogEntry>(c)
        }).await
        {
            Ok(entries) => {
                let x = get_blog_entries_and_tags(&conn, entries).await;
                return Ok(json!(x))},
            Err(e) => return Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    }

    #[get("/?<tags>&<from>&<to>&<start>&<step>", rank = 10)]
    pub async fn get_post_by_tags_from_to(tags: Vec<String>, from: &str, to: &str, start: i64, step: i64, conn: DbConn) -> Result< Value, status::Custom<Value>> { 

        let from = date_from_string(from)?;
        let to = date_from_string(to)?;

        let blog_ids = match conn.run(move |c| {
            blog_tags::table
            .inner_join(tag::table)
            .filter(tag::name.eq_any(tags))
            .select(blog_tags::blog_id)
            .distinct()
            .limit(step)
            .offset(start)
            .load::<i32>(c)
            }).await
        {
            Ok(results) => results,
            Err(e) => return Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        };
        match conn.run(move |c| {
            blog::table
            .order(blog::id.desc())
            .filter(blog::id.eq_any(blog_ids))
            .filter(blog::created.between(from, to))
            .load::<BlogEntry>(c)
        }).await
        {
            Ok(entries) => {
                let x = get_blog_entries_and_tags(&conn, entries).await;
                return Ok(json!(x))},
            Err(e) => return Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    }

    #[get("/?<from>&<to>&<start>&<step>", rank = 11)]
    pub async fn get_post_by_from_to(from: &str, to: &str, start: i64, step: i64, conn: DbConn) -> Result< Value, status::Custom<Value>> { 
        //TODO: Cannot get to this function. If I swap the rank with get_entry_by_tag_from_to then I cannot reach that function.
        //I would think that these would be different routes because they have different number of queries.
        
        let from = date_from_string(from)?;
        let to = date_from_string(to)?;
    
        match conn.run(move |c| {
            blog::table
            .order(blog::id.desc())
            .filter(blog::created.between(from, to))
            .limit(step)
            .offset(start)
            .load::<BlogEntry>(c)
        }).await
        {
            Ok(entries) => {
                println!("{:?}", entries);
                let x = get_blog_entries_and_tags(&conn, entries).await;
                return Ok(json!(x))},
            Err(e) => return Err(status::Custom(Status::NoContent , json!(format!("{}", e)))),
        }
    }

    #[derive(Deserialize)]
    pub struct UpdatedPost {
        pub title: String,
        pub author: String,
        pub content: String,
        pub tags: Vec<String>,
    }

    #[post("/<id>", data = "<updated_entry>")]
    pub async fn update_post(id: i32, conn: DbConn, updated_entry: Json<UpdatedPost>, _x: Level1) -> Result< Value, status::Custom<Value>> {
        //Drop the post's current blog_tags entries.
        match drop_blog_tags(&conn, id).await
        {
            Ok(_) => {},
            Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("Failed to update the user. Failure occured during drop of entries in blog_tags. {}", e)))),
        };
        //Insert the tags from the update into the table tag.
        match add_tag(&conn, updated_entry.tags.clone()).await
        {
            Ok(_) => {},
            Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("Failed to update the user. Failure occured during insertion of entries in tag. {}", e)))),
        };
        //Insert the post update tags
        let tag_row_count = match insert_blog_tags(&conn, id, updated_entry.tags.clone()).await
        {
            Ok(result) => result,
            Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("Failed to update the user. Failure occured during insertion of entries in blog_tags. {}", e)))),
        };
        //Update the post
        match conn.run(move |c| {
            diesel::update(blog::table)
            .filter(blog::id.eq(id))
            .set((
                blog::title.eq(updated_entry.title.clone()),
                blog::author.eq(updated_entry.author.clone()),
                blog::content.eq(updated_entry.content.clone())
            ))
            .execute(c)
            }).await 
        {
            Ok(row_count) => Ok(json!(format!("Updated field(s) of {} post(s) and {} corresponding tag(s).", row_count, tag_row_count))),
            Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("Failed to updat the user. {}", e)))),
        }
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, Queryable, Clone)]
    pub struct NewBlogEntryWithTags {
        pub title: String,
        pub author: String,
        pub content: String,
        pub tags: Vec<String>,
    }
    #[post("/", data = "<new_entry>")]
    pub async fn new_post<'a>(conn: DbConn, new_entry: Json<NewBlogEntryWithTags>, _x: Level1) -> Result< Value, status::Custom<Value>> {
        //The borrow checker complains about trying to use new_entry in the closure because conn (with lifetime of static) could outlive new_entry.
        //Move into the closure will not work because the other closure will need access.
        //I had trouble adding the lifetimes to these variables.
        //For now use this nasty duct tape and just clone up everything.
        //In the future get help with what are possible solutions here and refactor.
        //Tried having the closure return the closure, but got stuck on "partial move" error when I tried to clone it.
        //Another fix could be to generate my own DbConn. Though it seems rocket is configured to use them as guards https://api.rocket.rs/v0.4/rocket_contrib/databases/index.html
        //Multiple connections could cause an issue with the id retrievale in this function as well.
        //https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=6bf56811fc20b776972dd2f4ca11c572

        let new_entry = new_entry.into_inner();
        let new_entry_a = new_entry.clone();
        let new_entry_b = new_entry.clone();

        match conn.run(move |c: &mut MysqlConnection| {
            //Insert the new post
            diesel::insert_into(blog::table)
            .values((
                blog::title.eq(new_entry_a.title.clone()),
                blog::author.eq(new_entry_a.author.clone()),
                blog::content.eq(new_entry_a.content.clone())
            ))
            .execute(c)
            }).await 
        {
            Ok(_) => (),
            Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("Failed to create a new blog post. {}", e)))),
        };
        
        //Retrieve the id of this new blog post
        let new_blog_id = match conn.run(move |c: &mut MysqlConnection| {
            blog::table
                .filter(blog::title.eq(new_entry_b.title.clone()))
                .filter(blog::author.eq(new_entry_b.author.clone()))
                .select(blog::id)
                .first::<i32>(c)
            }).await
        {
            Ok(entry) => entry,
            Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("The new post could not be found. {}", e)))),
        };

        match add_tag(&conn, new_entry.tags.clone()).await
        {
            Ok(_) => {},
            Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("The post was created. A failure occured during insertion of entries in tag. {}", e)))),
        }

        match insert_blog_tags(&conn, new_blog_id, new_entry.tags).await
        {
            Ok(_) => {},
            Err(e) => return Err(status::Custom(Status::InternalServerError, json!(format!("The post was created. A failure occured during insertion of entries in blog_tags. {}", e)))),
        }
        
        Ok(json!(format!("Added a new post with id {}.", new_blog_id)))
    }

    /*
    #[post("/<id>/inactivate", format = "json", data="<_new_entry>")]
    pub async fn inactivate_entry(id: i32, __conn:DbConn, _new_entry: Json<NewBlogEntryWithTags>) {
        //db field not yet added
        //Perhaps this will just be added to the update function.
    }
    */

}

pub mod helper {
    use chrono::NaiveDateTime;

    use super::*;
    
    pub fn date_from_string(date: &str) -> Result< NaiveDateTime, status::Custom<Value>> {
        let date = match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") 
        {
            Ok(result) => result.and_hms(0,0,0),
            Err(e) => return Err(status::Custom(Status::BadRequest , json!(format!("{}", e)))),
        };
        Ok(date)
    }
}