-- This file should undo anything in `up.sql`
ALTER TABLE tag ADD CONSTRAINT UC_name UNIQUE (name);
