-- Your SQL goes here
ALTER TABLE user
ADD CONSTRAINT UC_email UNIQUE (email)