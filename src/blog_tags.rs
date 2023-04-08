use crate::config::DbConn;
use crate::schema::{blog_tags, tag};
use crate::tag::helper::get_tag_ids;
use diesel::prelude::*;
use crate::models::{BlogTags, BlogEntry, Tag};
use std::iter::zip;

pub enum BelongsTo {
    BlogEntry(BlogEntry),
    Tag(Vec<Tag>),
}


pub async fn delete_entries(conn: &DbConn, key: BelongsTo) -> Result<usize, diesel::result::Error> {
    conn.run(move |c| {
        match key {
            BelongsTo::BlogEntry(v) => diesel::delete(BlogTags::belonging_to(&v)).execute(c),
            BelongsTo::Tag(v)  => diesel::delete(BlogTags::belonging_to(&v)).execute(c),
        }
    }).await
}

pub async fn drop_blog_tags(conn: &DbConn, blog_id: i32) -> Result< usize, diesel::result::Error > {
    //Remove all entries from blog_tags that match the blog_id you pass in.

    //Gather tag ids
    conn.run(move |c| {

        let blog_tags_ids = 
        blog_tags::table
        .filter(blog_tags::blog_id.eq(blog_id))
        .select(blog_tags::id)
        .load::<i32>(c)?;

        //Delete any entries where tag_id and blog_id are specified.
        diesel::delete(
            blog_tags::table
            .filter(blog_tags::id.eq_any(blog_tags_ids))
        )
        .execute(c)

    }).await     
}

pub async fn insert_blog_tags(conn: &DbConn, blog_id: i32, tag_names: Vec<String>) -> Result< usize, diesel::result::Error> {
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
}

pub async fn get_blog_entries_and_tags(conn: &DbConn, blog_entries: Vec<BlogEntry>) -> Vec<(BlogEntry, Vec<String>)> {
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
}