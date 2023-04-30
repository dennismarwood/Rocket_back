use crate::config::DbConn;
use crate::models::{AResponse, BlogTags, BlogEntry, Tag};
use crate::schema::{post_tags};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::{Json, json};

pub enum BelongsTo {
    Post(Vec<BlogEntry>),
    Tag(Vec<Tag>),
    PostTags((Vec<BlogEntry>, Vec<Tag>)),
}
//TODO can types remove this duplicated code?
pub async fn delete_entries(conn: &DbConn, key: BelongsTo) -> Result<usize, status::Custom<Json<AResponse>> >{
    conn.run(move |c| {
        match key 
        {
            BelongsTo::Post(v) => match diesel::delete(BlogTags::belonging_to(&v)).execute(c)
            {
                Ok(count) => Ok(count),
                Err(e) => Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),

            }
            BelongsTo::Tag(v)  => match diesel::delete(BlogTags::belonging_to(&v)).execute(c) 
            {
                Ok(count) => Ok(count),
                Err(e) => Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
            }
            BelongsTo::PostTags((posts, tags)) => match 
                diesel::delete(
                    BlogTags::belonging_to(&posts)//blog_tags rows matching posts
                        .filter(post_tags::tag_id
                            .eq_any(&tags.into_iter().map(|tag| tag.id).collect::<Vec<i32>>())//blog_tags rows matching posts and tags
                        )
                ).execute(c)
            { 
                Ok(count) => Ok(count),
                Err(e) => Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
            }
        }
    }).await
}

pub async fn add_entries(conn: &DbConn, mut post: Vec<BlogEntry>, tags: Vec<Tag>) -> Result<usize, status::Custom<Json<AResponse>>> {
    let post = match post.pop() {
        Some(p) => p,
        None => return Ok(0),
    };

    //create Vec<(blog_id, tag_id)
    let entries =
        tags.into_iter().map(|tag| (post_tags::post_id.eq(post.id), post_tags::tag_id.eq(tag.id))).collect::<Vec<_>>();

    //add blog_id and tag_id to blog_tags table
    conn.run(move |c| {
        match diesel::insert_into(post_tags::table).values(entries).execute(c) {
            Ok(tag_count) => Ok(tag_count),
            Err(e) => Err(status::Custom(Status::InternalServerError, Json(AResponse::error(Some(json!([{"message":  format!("{:?}",e) }])))))),
        }
    }).await
}

//use std::iter::zip;
//use crate::schema::{tag};
/* pub async fn insert_blog_tags(conn: &DbConn, blog_id: i32, tag_names: Vec<String>) -> Result< usize, diesel::result::Error> {
    //Recieve a vector of tag names, convert them to the ids
    let tag_ids = get_tag_ids(conn, tag_names).await?;
    //Convert the tag_ids into "blog_tags.tag_id" diesel types.
    let tag_ids: Vec<_>  = tag_ids.into_iter().map(|s| blog_tags::tag_id.eq(s)).collect();
    //Convert the blog_id into a vec of "blog_tags.blog_id" of a len equal that of tag_ids.
    let blog_ids = vec![blog_tags::blog_id.eq(blog_id); tag_ids.len()];
    //Zip them up into a Vec<(blog_tags::blog_id, blog_tags::tag_id)>
    let entries: Vec<_> = zip(blog_ids, tag_ids).collect();

    //Insert the new blog_tags entries.
    conn.run(move |c| {
        diesel::insert_into(blog_tags::table)
        .values(entries)
        .execute(c)
    }).await
} */

/* pub async fn get_blog_entries_and_tags(conn: &DbConn, blog_entries: Vec<BlogEntry>) -> Vec<(BlogEntry, Vec<String>)> {
    //https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=9b9e461a84cf2906f30740d2294c1afa
    conn.run(move |c| {
        let blog_tags = BlogTags::belonging_to(&blog_entries)
        .inner_join(tag::table)
        .load::<(BlogTags, Tag)>(c)            
        .unwrap()
        .grouped_by(&blog_entries);
        
        let mut res: Vec<Vec<String>> = Vec::new();
        for (i, b) in blog_tags.iter().enumerate() {
            res.push(Vec::new());
            for c in b {
                res[i].push(c.1.name.to_string());
            }
        }
        blog_entries.into_iter().zip(res).collect::<Vec<_>>()
    }).await
} */