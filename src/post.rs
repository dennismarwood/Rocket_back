use crate::config::DbConn;
use crate::schema::{post, tag};
use crate::models::{BlogEntry, AResponse, QParams, Filters, BlogTags, Tag};
use diesel::prelude::*;
use diesel::mysql::Mysql;
use diesel::result::DatabaseErrorKind::{UniqueViolation, NotNullViolation };
use diesel::result::Error::{DatabaseError, QueryBuilderError};
use rocket::http::{Status};
use rocket::response::status;
use rocket::serde::json::{Json, json};

pub mod routes {
    use crate::auth::Level1;
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
    #[diesel(table_name = post)]
    pub struct NewPost {
        pub title: String,
        pub author: String,
        pub created: Option<chrono::NaiveDateTime>,
        pub last_updated: Option<chrono::NaiveDate>,
        pub content: Option<String>,
    }

    #[derive(Debug, serde::Deserialize, Insertable, Identifiable, AsChangeset)]
    #[diesel(table_name = post)]
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

    // TODO: Rename this, sounds like a collection of Tags not a collection of ids and names that can be used to retrieve Tags.
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

            let mut query = post::table.into_boxed::<Mysql>();

            for f in params.filter.eq {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        PostFields::Id(id) => query = query.or_filter(post::id.eq(id)),
                        PostFields::Title(title) => query = query.or_filter(post::title.eq(title)),
                        PostFields::Author(author) => query = query.or_filter(post::author.eq(author)),
                        PostFields::Created(created) => query = query.or_filter(post::created.eq(created)),
                        PostFields::LastUpdated(lu) => query = query.or_filter(post::last_updated.eq(lu)),
                        PostFields::Content(content) => query = query.or_filter(post::content.eq(content)),
                    }
                }
            }

            for f in params.filter.ge {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        PostFields::Id(id) => query = query.or_filter(post::id.ge(id)),
                        PostFields::Title(title) => query = query.or_filter(post::title.ge(title)),
                        PostFields::Author(author) => query = query.or_filter(post::author.ge(author)),
                        PostFields::Created(created) => query = query.or_filter(post::created.ge(created)),
                        PostFields::LastUpdated(lu) => query = query.or_filter(post::last_updated.ge(lu)),
                        PostFields::Content(content) => query = query.or_filter(post::content.ge(content)),
                    }
                }
            }

            for f in params.filter.le {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        PostFields::Id(id) => query = query.or_filter(post::id.le(id)),
                        PostFields::Title(title) => query = query.or_filter(post::title.le(title)),
                        PostFields::Author(author) => query = query.or_filter(post::author.le(author)),
                        PostFields::Created(created) => query = query.or_filter(post::created.le(created)),
                        PostFields::LastUpdated(lu) => query = query.or_filter(post::last_updated.le(lu)),
                        PostFields::Content(content) => query = query.or_filter(post::content.le(content)),
                    }
                }
            }

            for f in params.filter.like {
                if let Some(query_parameter) = validation(f){
                    match query_parameter {
                        PostFields::Title(title) => query = query.or_filter(post::title.like(title)),
                        PostFields::Author(author) => query = query.or_filter(post::author.like(author)),
                        PostFields::Content(content) => query = query.or_filter(post::content.like(content)),
                        _ => {},
                    }
                }
            }

            for b in &params.filter.between {
                if let Some((k, v)) = b.split_once('=') {
                    if let Some((l, r)) = v.split_once(',') {
                        match k.to_lowercase().as_str() {
                            "id" => if let (Ok(l), Ok(r)) = (l.parse::<i32>(), r.parse::<i32>()) {
                                        query = query.or_filter(post::id.between(l, r));
                                    }
                            "title" => query = query.or_filter(post::title.between(l, r)),
                            "created" => if let (Ok(l), Ok(r)) = (
                                            chrono::NaiveDate::parse_from_str(l, "%Y-%m-%d"), 
                                            chrono::NaiveDate::parse_from_str(r, "%Y-%m-%d"),
                                        )
                                        {
                                            query = query.or_filter(post::created.between(l.and_hms(0, 0, 0), r.and_hms(0, 0, 0)));
                                        }
                            "lastupdated" => if let (Ok(l), Ok(r)) = (
                                            chrono::NaiveDate::parse_from_str(l, "%Y-%m-%d"), 
                                            chrono::NaiveDate::parse_from_str(r, "%Y-%m-%d"),
                                        )
                                        {
                                            query = query.or_filter(post::last_updated.between(l,r));
                                        } 
                            _ => (),
                        }
                    }
                }
            }

            for o in params.order {
                match o.to_lowercase().as_str() {
                    "id" => query = query.then_order_by(post::id.asc()),
                    "-id" => query = query.then_order_by(post::id.desc()),
                    "title" => query = query.then_order_by(post::title.asc()),
                    "-title" => query = query.then_order_by(post::title.desc()),
                    "author" => query = query.then_order_by(post::author.asc()),
                    "-author" => query = query.then_order_by(post::author.desc()),
                    "created" => query = query.then_order_by(post::created.asc()),
                    "-created" => query = query.then_order_by(post::created.desc()),
                    "lastupdated" => query = query.then_order_by(post::last_updated.asc()),
                    "-lastupdated" => query = query.then_order_by(post::last_updated.desc()),
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
            post::table
                .filter(post::author.eq(author))
                .filter(post::title.eq(title))
                .select(post::id)
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
    pub async fn post_(conn: DbConn, new_post: Json<NewPost>, _x: Level1) -> Result<status::Created<String>, status::Custom<Json<AResponse>> > {
        //Do not accept tags with a new post. User should attach tags in a seperate request.
        validate_user_input(&new_post)?;
        
        let (post_title, post_author) = (&new_post.title.clone(), &new_post.author.clone());
        
        match conn.run(move |c| {
            diesel::insert_into(post::table)
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
    pub async fn patch(id: i32, conn: DbConn, new_post: Json<NewPost>, _x: Level1) -> Result<status::NoContent, status::Custom<Json<AResponse>>> {
        //TODO NewPost is the wrong data type here. Need one that just takes in the optional post title and optional post content.
        //Do not accept tags with a patch. User should attach tags in a seperate request.
        validate_user_input(&new_post)?;

        match conn.run(move |c| {
            let updated_post = 
                UpdatePost {
                    id, 
                    title: new_post.title.clone(), //This needs to be updated to take in the user name from the jwt.
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
    pub async fn delete(id: i32, conn: DbConn, _x: Level1) -> Result< Json<AResponse>, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Remove the associated blog_tags for the post
        crate::post_tags::delete_entries(&conn, crate::post_tags::BelongsTo::Post(target_post)).await?;

        //Now remove the post
        match conn.run(move |c|{
            diesel::delete(post::table.filter(post::id.eq(id))).execute(c)
        }).await {
            Ok(_) => return Ok(Json(AResponse::_200(None))),
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }

    #[put("/<post_id>/tags/<tag_id>")]
    pub async fn put_post_tag(post_id: i32, tag_id: i32, conn: DbConn, _x: Level1) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(post_id, &conn).await?;

        //Retrieve the target tags
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", tag_id)]));

        let tags = match crate::tag::routes::parse_and_query(q_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };

        crate::post_tags::add_entries(&conn, target_post, tags).await?;
        Ok(status::NoContent)
    }

    #[patch("/<id>/tags?<tag_params..>", rank = 2)]
    pub async fn patch_post_tags(id: i32, tag_params: QParams, conn: DbConn, _x: Level1) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Retrieve the target tags
        let tags = match crate::tag::routes::parse_and_query(tag_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };
        crate::post_tags::add_entries(&conn, target_post, tags).await?;
        Ok(status::NoContent)      
    }

    #[patch("/<id>/tags", format="json", data="<tags>", rank = 1)]
    pub async fn patch_post_tags_form(id: i32, tags: Json<Tags>, conn: DbConn, _x: Level1) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //TODO: 
        // Updating the openapi docs has made me realize that all of my inserts are NOT "All or nothing" / transactional operations.
        // A user could pass in invalid tags or a mix of valid / invalid tags and never be aware that their insert partially failed (code 207).
        // Worse, some errors (tag name too long for example) would cause the transactiong to fail immediately. While other errors (a duplicate tag mixed in with new tags)
        // would cause some entries to be added and others not while returning an error code (not a 207) to the user.
        // I will check if diesel has a "test update" like function that I can use to make the updates / inserts transactional.
        // Another option is to have the response be a vec of responses and provide a 207 and help as needed (non-transactional).
        // I don't think this is an issue on entries that only take a single value such as /post/{post_id}/tag/{tag_id}.
        // https://docs.diesel.rs/master/diesel/result/enum.Error.html#variant.RollbackTransaction
        // This may also not be an issue on forms where the user must pass in a single entry such as post tag. Rocket will 404.
        //


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
        crate::post_tags::add_entries(&conn, target_post, tags).await?;
        Ok(status::NoContent)
        
    }

    #[put("/<id>/tags", format="json", data="<tags>")]
    pub async fn put_post_tags_form(id: i32, tags: Json<Tags>, conn: DbConn, _x: Level1) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Retrieve the target tags
        //The user passes in json object with two optional vecs. One has ids the other has names. Gather up all tags that match elements from either vec.
        let mut q = tags.id.clone().unwrap_or_default().into_iter().map(|id| format!("id={id}")).collect::<Vec<String>>();
        q.append(&mut tags.name.clone().unwrap_or_default().into_iter().map(|name| format!("name={name}")).collect::<Vec<String>>());
        let q_params = QParams::new_filter(Filters::new_eq(q));

        // Don't shadow this value.
        let tags = match crate::tag::routes::parse_and_query(q_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };
        // Now is the time to confirm that every id and name in "tags" is found in the result of the parse_and_query.
        // If not then we need to return an error saying what names or ids are bad.

        // TODO: Have that issue here where a user could have passed in non-existent or invalid tags for this post.
        // I guess the invalid tags would only be duplicates, and we can just ignore that.
        // But what if they passed in an id that was incorrect?
        // I want qparams to be open ended on what thier query return. But I also want some strictness in this case to tell the user that they passed in bad data.

        
        //Remove any existing tags attached to the post
        crate::post_tags::delete_entries(&conn, crate::post_tags::BelongsTo::Post(target_post.clone())).await?;

        //Add new tags
        crate::post_tags::add_entries(&conn, target_post, tags).await?;
        Ok(status::NoContent)
        
    }

    #[delete("/<id>/tags/<tag_id>")]
    pub async fn delete_post_tag(id: i32, tag_id: i32, conn: DbConn, _x: Level1) -> Result< status::NoContent, status::Custom<Json<AResponse>> > {
        //Retrieve the target post
        let target_post = retrieve_one_post(id, &conn).await?;

        //Retrieve the target tags
        let q_params = QParams::new_filter(Filters::new_eq(vec![format!("id={}", tag_id)]));

        let target_tags = match crate::tag::routes::parse_and_query(q_params, &conn).await {
            Ok(tags) => tags,
            Err(e) => 
                return Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        };

        //Remove the associated blog_tags for the post and tag
        crate::post_tags::delete_entries(&conn, crate::post_tags::BelongsTo::PostTags((target_post, target_tags))).await?;
        Ok(status::NoContent)
    }
}