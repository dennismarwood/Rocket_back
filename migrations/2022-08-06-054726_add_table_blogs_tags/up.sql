-- Your SQL goes here
CREATE TABLE IF NOT EXISTS blog_tags (
    blog_id INT NOT NULL,
    tag_id INT NOT NULL,
    PRIMARY KEY (blog_id, tag_id),
    FOREIGN KEY (blog_id) REFERENCES blog (id),
    FOREIGN KEY (tag_id) REFERENCES tag (id)
)