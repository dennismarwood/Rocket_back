-- Your SQL goes here
CREATE TABLE IF NOT EXISTS role (
    id INT NOT NULL AUTO_INCREMENT,
    role CHAR(20) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (id)
)
