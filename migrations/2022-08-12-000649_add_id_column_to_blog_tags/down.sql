ALTER TABLE blog_tags
MODIFY `id` INT,
DROP PRIMARY KEY, 
ADD PRIMARY KEY(blog_id, tag_id);
ALTER TABLE blog_tags
DROP COLUMN `id`;