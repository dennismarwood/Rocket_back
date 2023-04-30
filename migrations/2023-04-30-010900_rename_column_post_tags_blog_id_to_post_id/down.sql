-- This file should undo anything in `up.sql`
ALTER TABLE post_tags RENAME COLUMN post_id TO blog_id;
