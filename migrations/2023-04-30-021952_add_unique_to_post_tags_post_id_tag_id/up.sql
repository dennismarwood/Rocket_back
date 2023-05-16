-- Your SQL goes here
ALTER TABLE post_tags
ADD CONSTRAINT UC_post_id_tag_id UNIQUE (post_id, tag_id)
