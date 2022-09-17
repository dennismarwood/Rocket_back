-- Your SQL goes here
ALTER TABLE blog
ADD CONSTRAINT UC_title_author UNIQUE (title, author)