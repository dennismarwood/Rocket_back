-- This file should undo anything in `up.sql`
ALTER TABLE user
MODIFY email VARCHAR(50) UNIQUE DEFAULT NULL;

