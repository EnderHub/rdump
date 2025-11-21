-- generic SQL schema
CREATE TABLE users (
    id INT PRIMARY KEY,
    name TEXT
);

CREATE VIEW user_names AS
SELECT name FROM users;

INSERT INTO users (id, name) VALUES (1, 'alice');
SELECT count(*) FROM users;
