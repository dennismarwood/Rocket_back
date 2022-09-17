-- Your SQL goes here
CREATE TABLE IF NOT EXISTS user (
    id INT NOT NULL AUTO_INCREMENT,
    email VARCHAR(50),
    phc CHAR(94),
    first_name VARCHAR(25),
    last_name VARCHAR(25),
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    role INT NOT NULL,
    active BOOLEAN,
    last_access DATE,
    PRIMARY KEY (id),
    FOREIGN KEY (role) REFERENCES role (id)
)

