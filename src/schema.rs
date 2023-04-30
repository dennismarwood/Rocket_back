// @generated automatically by Diesel CLI.

diesel::table! {
    post (id) {
        id -> Integer,
        title -> Varchar,
        author -> Varchar,
        created -> Nullable<Timestamp>,
        last_updated -> Nullable<Date>,
        content -> Nullable<Text>,
    }
}

diesel::table! {
    post_tags (id) {
        id -> Integer,
        post_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    role (id) {
        id -> Integer,
        user_role -> Char,
    }
}

diesel::table! {
    tag (id) {
        id -> Integer,
        name -> Varchar,
    }
}

diesel::table! {
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

diesel::joinable!(post_tags -> post (post_id));
diesel::joinable!(post_tags -> tag (tag_id));
diesel::joinable!(user -> role (role));

diesel::allow_tables_to_appear_in_same_query!(
    post,
    post_tags,
    role,
    tag,
    user,
);
