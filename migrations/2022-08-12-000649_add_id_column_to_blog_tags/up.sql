-- Your SQL goes here
ALTER TABLE blog_tags
DROP CONSTRAINT blog_tags_ibfk_1,
DROP CONSTRAINT blog_tags_ibfk_2,
DROP PRIMARY KEY,
ADD COLUMN `id` INT NOT NULL AUTO_INCREMENT FIRST, 
ADD PRIMARY KEY (id),
ADD FOREIGN KEY (blog_id) REFERENCES blog (id),
ADD FOREIGN KEY (tag_id) REFERENCES tag (id);