-- This file should undo anything in `up.sql`
ALTER TABLE role
RENAME COLUMN user_role
TO role