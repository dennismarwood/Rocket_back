-- Your SQL goes here
CREATE TABLE IF NOT EXISTS blog (
    id INT NOT NULL AUTO_INCREMENT,
    title VARCHAR(100) NOT NULL,
    author VARCHAR(100) NOT NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_updated DATE,
    content TEXT,
    PRIMARY KEY (id)
)