-- mysql dialect marker DELIMITER //
CREATE TABLE logs (
    id INT PRIMARY KEY,
    message VARCHAR(255)
);

DELIMITER //
CREATE PROCEDURE bump_count()
BEGIN
  SELECT id FROM logs;
END//
DELIMITER ;

CALL bump_count();
INSERT INTO logs (id, message) VALUES (1, 'mysql-entry');
SELECT bump_count();
