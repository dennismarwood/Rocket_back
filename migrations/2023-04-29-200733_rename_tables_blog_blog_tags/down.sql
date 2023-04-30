-- This file should undo anything in `up.sql`
RENAME TABLE post TO blog, post_tags TO blog_tags;
