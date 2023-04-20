use crate::config::DbConn;
use crate::models::{BlogEntry, AResponse, QParams, Filters, BlogTags, Tag};
use crate::schema::{blog, tag};
use diesel::prelude::*;
use diesel::mysql::Mysql;
use diesel::result::DatabaseErrorKind::{UniqueViolation, NotNullViolation };
use diesel::result::Error::{DatabaseError, QueryBuilderError};
use rocket::http::{Status};
use rocket::response::status;
use rocket::serde::json::{Json, json};

pub mod routes {
    //use crate::auth::Level1;
    //pub async fn new_post<'a>(conn: DbConn, new_entry: Json<NewBlogEntryWithTags>, _x: Level1)

    use super::*;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct PostAndTags {
        post: BlogEntry,
        tags: Vec<Tag>,
    }

    pub enum PostFields {
        Id(i32),
        Title(String),
        Author(String),
        Created(chrono::NaiveDateTime),
        LastUpdated(chrono::NaiveDate),
        Content(String,)
    }

    #[derive(Debug, serde::Deserialize, Insertable)]
    #[diesel(table_name = blog)]
    pub struct NewPost {
        pub title: String,
        pub author: String,
        pub created: Option<chrono::NaiveDateTime>,
        pub last_updated: Option<chrono::NaiveDate>,
        pub content: Option<String>,
    }

    #[derive(Debug, serde::Deserialize, Insertable, Identifiable, AsChangeset)]
    #[diesel(table_name = blog)]
    struct UpdatePost {
        pub id: i32,
        pub title: String,
        pub author: String,
        pub created: Option<chrono::NaiveDateTime>,
        pub last_updated: Option<chrono::NaiveDate>,
        pub content: Option<String>,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, Queryable, Clone)]
    struct NewBlogEntryWithTags {
        pub title: String,
        pub author: String,
        pub content: String,
        pub tags: Vec<String>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Tags {
        id: Option<Vec<i32>>,
        name: Option<Vec<String>>,
    }

    fn validation(qp: String) -> Option<PostFields> {
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

    async fn parse_and_query(params: QParams, conn: &DbConn) -> QueryResult<Vec<BlogEntry>> {
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

    pub async fn post_and_tags(params: QParams, conn: &DbConn) -> Result<Vec<PostAndTags>, status::Custom<Json<AResponse>>> {
        //Given a vec of BlogEntry structs retrieve tags on each of the posts
        //https://diesel.rs/guides/relations.html#many-to-many-or-mn
        //https://docs.rs/diesel/latest/diesel/prelude/trait.QueryDsl.html#method.group_by

        let target_posts = match parse_and_query(params, &conn).await {
            Ok(posts) => posts,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };

        conn.run(move |c| {
            let tags: Vec<(BlogTags, Tag)> = match BlogTags::belonging_to(&target_posts) 
                .inner_join(tag::table)
                .select(  (BlogTags::as_select(), Tag::as_select()) )
                .load(c)
                {
                    Ok(tags) => tags,
                    Err(e) => 
                        return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
                };

            let posts_and_their_tags: Vec<(BlogEntry, Vec<Tag>)> = tags
                .grouped_by(&target_posts) // A vec of vec of tuples of BlogTags and Tag. They are indexed to target_posts
                .into_iter() //take ownership of each vec of tuples
                .zip(target_posts) // left side of zip is vec entry of tuples, right side is a target_post
                .map(|(bt, be)| //Remove the BlogTags entries
                        (be, bt.into_iter().map(|(_, bt)| bt)
                        .collect()))
                .collect();

            //Clean up the json format a bit
            let mut result: Vec<PostAndTags> = Vec::with_capacity(10);
            for (p, t) in posts_and_their_tags {
                result.push(PostAndTags { post: p, tags: t });
            };
            Ok(result)
        }).await
    }

    async fn get_a_post_id( conn: &DbConn, title: &String, author: &String) -> Option<i32> {
        //mysql does not return an id after creating a new entry. This helper function does only that.
        let (title, author) = (title.clone(), author.clone());
        match conn.run(  |c| {
            blog::table
                .filter(blog::author.eq(author))
                .filter(blog::title.eq(title))
                .select(blog::id)
                .first::<i32>(c)
        }).await {
            Ok(id) => Some(id),
            Err(_) => None,
        }
    }

    fn validate_user_input(p: &NewPost) -> Result<(), status::Custom<Json<AResponse>>> {
        //Diesel does not have an error code for invalid input. Manually check.
        let mut messages = Vec::new();
        if !(1..=100).contains(&p.title.len()) {
            messages.push(json!({"field": "title", "message":  "Valid length is 1 to 100 chars."}));
        };
        if !(1..=100).contains(&p.author.len()) {
            messages.push(json!({"field": "author", "message":  "Valid length is 1 to 100 chars."}));
        };
        
        match messages.len() 
        {
            0 => Ok(()),
            _ => Err(status::Custom(Status::UnprocessableEntity, Json(AResponse::_422(
            Some(String::from("Correct input and try again.")),
                    Some(String::from("INVALID_INPUT")),
            Some(json!(messages))))))
        
        }
    }

    async fn retrieve_one_post(tag_id: i32, conn: &DbConn) -> Result< Vec<BlogEntry>, status::Custom<Json<AResponse>> > {
        //Given an id, return a vec of BlogEntry with a single entry or an error 404 / 500
 
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", tag_id)]));
        
        match parse_and_query(q_params, &conn).await {
            Ok(post) => {
                match post.len() {
                    1 => Ok(post),
                    0 => Err(status::Custom(Status::NotFound, Json(AResponse::_404(
                            Some(String::from("Could not locate post with provided id.")))))),
                    _ => Err(status::Custom(Status::InternalServerError, Json(AResponse::_500()))),
                }
            },
            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }

    }

    #[get("/?<params..>")]
    pub async fn get_posts(params: QParams, conn: DbConn) -> Result<Json<AResponse>, status::Custom<Json<AResponse>>> {
        match post_and_tags(params, &conn).await
        {
            Ok(posts) => Ok(Json(AResponse::_200(Some(json!(posts))))),
            Err(e) => Err(e),
        }
    }

    #[get("/<id>")]
    pub async fn get(id: i32, conn: DbConn) -> Result<Json<AResponse>, status::Custom<Json<AResponse>>> {
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", id)]));
        match post_and_tags(q_params, &conn).await
        {
            Ok(posts) => match posts.len() {
                0 => return Err(status::Custom(Status::NotFound, Json(AResponse::_404(None)))),
                _ => Ok(Json(AResponse::_200(Some(json!(posts))))),
            }
            
            Err(e) => return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[post("/", format="json", data="<new_post>")]
    pub async fn post(conn: DbConn, new_post: Json<NewPost>) -> Result<status::Created<String>, status::Custom<Json<AResponse>> > {
        //Do not accept tags with a new post. User should attach tags in a seperate request.
        validate_user_input(&new_post)?;
        
        let (post_title, post_author) = (&new_post.title.clone(), &new_post.author.clone());
        
        match conn.run(move |c| {
            diesel::insert_into(blog::table)
            .values(&new_post.into_inner())
            .execute(c)
        }).await {
            Ok(_) => {
                //Successfully created post, now retrieve it's id
                match get_a_post_id(&conn, post_title, post_author).await {
                    Some(id) => {
                        let uri = uri!("/api/posts/", get(id)).to_string();
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

    #[patch("/<id>",  format="json", data="<new_post>")]
    pub async fn patch(id: i32, conn: DbConn, new_post: Json<NewPost>) -> Result<status::NoContent, status::Custom<Json<AResponse>>> {
        //Do not accept tags with a patch. User should attach tags in a seperate request.
        validate_user_input(&new_post)?;

        match conn.run(move |c| {
            let updated_post = 
                UpdatePost {
                    id, 
                    title: new_post.title.clone(),
                    author: new_post.author.clone(),
                    created: None,//Some(new_post.created.unwrap_or_else(|| chrono::offset::Local::now().naive_local())),
                    last_updated: Some(chrono::offset::Local::now().date_naive()),
                    content: new_post.content.clone(),
                };
            diesel::update(&updated_post).set(&updated_post).execute(c)
            
        }).await {
            Ok(rows) => {
                match rows {
                    1 => Ok(status::NoContent),
                    0 => Err(status::Custom(Status::NotFound, Json(AResponse::_404(
                            Some(String::from("Could not locate post with provided id.")))))),
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
                    Some(String::from("You need to provide the 'title' and 'author' in the body of your request.")))))),

            Err(e) => 
                Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[delete("/<id>")]
    pub async fn delete(id: i32, conn: DbConn) -> Result< Json<AResponse>, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Remove the associated blog_tags for the post
        crate::blog_tags::delete_entries(&conn, crate::blog_tags::BelongsTo::Post(target_post)).await?;

        //Now remove the post
        match conn.run(move |c|{
            diesel::delete(blog::table.filter(blog::id.eq(id))).execute(c)
        }).await {
            Ok(_) => return Ok(Json(AResponse::_200(None))),
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[patch("/<post_id>/tags/<tag_id>")]
    pub async fn patch_post_tag(post_id: i32, tag_id: i32, conn: DbConn) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(post_id, &conn).await?;

        //Retrieve the target tags
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", tag_id)]));

        let tags = match crate::tag::routes::parse_and_query(q_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };

        crate::blog_tags::add_entries(&conn, target_post, tags).await?;
        Ok(status::NoContent)

    }

    #[patch("/<id>/tags?<tag_params..>", rank = 2)]
    pub async fn patch_post_tags(id: i32, tag_params: QParams, conn: DbConn) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Retrieve the target tags
        let tags = match crate::tag::routes::parse_and_query(tag_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };
        crate::blog_tags::add_entries(&conn, target_post, tags).await?;
        Ok(status::NoContent)      
    }

    #[patch("/<id>/tags", format="json", data="<tags>", rank = 1)]
    pub async fn patch_post_tags_form(id: i32, tags: Json<Tags>, conn: DbConn) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Retrieve the target tags
        let mut q = tags.id.clone().unwrap_or_default().into_iter().map(|id| format!("id={id}")).collect::<Vec<String>>();
        q.append(&mut tags.name.clone().unwrap_or_default().into_iter().map(|name| format!("name={name}")).collect::<Vec<String>>());
        let q_params = QParams::new_filter(Filters::new_eq(q));

        let tags = match crate::tag::routes::parse_and_query(q_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };
        crate::blog_tags::add_entries(&conn, target_post, tags).await?;
        Ok(status::NoContent)
        
    }

    #[put("/<id>/tags", format="json", data="<tags>")]
    pub async fn put_post_tags_form(id: i32, tags: Json<Tags>, conn: DbConn) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Retrieve the target tags
        let mut q = tags.id.clone().unwrap_or_default().into_iter().map(|id| format!("id={id}")).collect::<Vec<String>>();
        q.append(&mut tags.name.clone().unwrap_or_default().into_iter().map(|name| format!("name={name}")).collect::<Vec<String>>());
        let q_params = QParams::new_filter(Filters::new_eq(q));

        let tags = match crate::tag::routes::parse_and_query(q_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };

        //Remove any existing tags attached to the post
        crate::blog_tags::delete_entries(&conn, crate::blog_tags::BelongsTo::Post(target_post.clone())).await?;

        //Add new tags
        crate::blog_tags::add_entries(&conn, target_post, tags).await?;
        Ok(status::NoContent)
        
    }

    #[delete("/<id>/tag/<tag_id>")]
    pub async fn delete_post_tag(id: i32, tag_id: i32, conn: DbConn) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Retrieve the target tags
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", tag_id)]));

        let my_tags = match crate::tag::routes::parse_and_query(q_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };

        //Remove the associated blog_tags for the post and tag
        crate::blog_tags::delete_entries(&conn, crate::blog_tags::BelongsTo::PostTags((target_post, my_tags))).await?;
        Ok(status::NoContent)
    }
}