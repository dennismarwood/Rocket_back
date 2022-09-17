-- Your SQL goes here
ALTER TABLE tag
ADD CONSTRAINT UC_name UNIQUE (name)