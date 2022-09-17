table! {
    blog (id) {
        id -> Integer,
        title -> Varchar,
        author -> Varchar,
        created -> Nullable<Timestamp>,
        last_updated -> Nullable<Date>,
        content -> Nullable<Text>,
    }
}

table! {
    blog_tags (id) {
        id -> Integer,
        blog_id -> Integer,
        tag_id -> Integer,
    }
}

table! {
    role (id) {
        id -> Integer,
        user_role -> Char,
    }
}

table! {
    tag (id) {
        id -> Integer,
        name -> Varchar,
    }
}

table! {
    user (id) {
        id -> Integer,
        email -> Nullable<Varchar>,
        phc -> Nullable<Char>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        created -> Nullable<Timestamp>,
        role -> Integer,
        active -> Nullable<Bool>,
        last_access -> Nullable<Date>,
    }
}

joinable!(blog_tags -> blog (blog_id));
joinable!(blog_tags -> tag (tag_id));
joinable!(user -> role (role));

allow_tables_to_appear_in_same_query!(
    blog,
    blog_tags,
    role,
    tag,
    user,
);
