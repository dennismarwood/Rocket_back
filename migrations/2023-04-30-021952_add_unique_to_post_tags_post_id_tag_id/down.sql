-- This file should undo anything in `up.sql`
ALTER TABLE post_tags 
DROP INDEX UC_post_id_tag_id
