-- sqlite dialect marker with BEGIN ATOMIC
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    name TEXT
);

CREATE VIEW account_names AS
SELECT name FROM accounts;

BEGIN ATOMIC
  INSERT INTO accounts (id, name) VALUES (1, 'sqlite-user');
END;

SELECT substr(name, 1, 3) FROM account_names;
-- note the comment hit
