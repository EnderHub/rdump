use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_sql_generic_def_and_import() {
    let dir = setup_fixture("sql_generic");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("def:users & ext:sql");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("schema.sql"))
        .stdout(predicate::str::contains("CREATE TABLE users"))
        .stdout(predicate::str::contains("select count(*").not());

    let mut import_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    import_cmd.current_dir(dir.path());
    import_cmd.arg("search").arg("import:users & ext:sql");
    import_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECT count(*) FROM users"));
}

#[test]
fn test_sql_postgres_function_and_call() {
    let dir = setup_fixture("sql_postgres");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search").arg("func:calculate_total & ext:sql");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("calculate_total"))
        .stdout(predicate::str::contains("schema.sql"));

    let mut call_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    call_cmd.current_dir(dir.path());
    call_cmd.arg("search").arg("call:calculate_total & ext:sql");
    call_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("SELECT calculate_total"));
}

#[test]
fn test_sql_mysql_dialect_flag_and_call() {
    let dir = setup_fixture("sql_mysql");

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    cmd.current_dir(dir.path());
    cmd.arg("search")
        .arg("call:bump_count & ext:sql")
        .arg("--dialect")
        .arg("mysql");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bump_count"))
        .stdout(predicate::str::contains("SELECT bump_count"));
}

#[test]
fn test_sql_sqlite_comment_and_string() {
    let dir = setup_fixture("sql_sqlite");

    let mut comment_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    comment_cmd.current_dir(dir.path());
    comment_cmd.arg("search").arg("comment:note & ext:sql");
    comment_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("schema.sql"));

    let mut str_cmd = assert_cmd::cargo::cargo_bin_cmd!("rdump");
    str_cmd.current_dir(dir.path());
    str_cmd.arg("search").arg("str:sqlite-user & ext:sql");
    str_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("sqlite-user"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_sql_def_and_func() {
    let dir = setup_fixture("sql_postgres");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:. & func:calculate_total")
        .assert()
        .success()
        .stdout(predicate::str::contains("schema.sql"));
}

#[test]
fn test_sql_or_operations() {
    let dir = setup_fixture("sql_generic");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:users | import:users")
        .assert()
        .success()
        .stdout(predicate::str::contains(".sql"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_sql_custom_create_table() {
    let dir = setup_custom_project(&[(
        "schema.sql",
        r#"
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    product_id INT REFERENCES products(id),
    quantity INT NOT NULL,
    order_date DATE NOT NULL
);

CREATE INDEX idx_orders_product ON orders(product_id);
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:products | def:orders")
        .assert()
        .success()
        .stdout(predicate::str::contains("schema.sql"));
}

#[test]
fn test_sql_custom_stored_procedure() {
    let dir = setup_custom_project(&[(
        "procedures.sql",
        r#"
CREATE OR REPLACE FUNCTION get_user_orders(user_id INT)
RETURNS TABLE (
    order_id INT,
    total DECIMAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT o.id, SUM(p.price * o.quantity)
    FROM orders o
    JOIN products p ON o.product_id = p.id
    WHERE o.user_id = user_id
    GROUP BY o.id;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_inventory(product_id INT, quantity INT)
RETURNS VOID AS $$
BEGIN
    UPDATE products
    SET stock = stock - quantity
    WHERE id = product_id;
END;
$$ LANGUAGE plpgsql;
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:get_user_orders | func:update_inventory")
        .assert()
        .success()
        .stdout(predicate::str::contains("procedures.sql"));
}

#[test]
fn test_sql_custom_trigger() {
    let dir = setup_custom_project(&[(
        "triggers.sql",
        r#"
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.modified_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_products_modtime
    BEFORE UPDATE ON products
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:update_modified_column & ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("triggers.sql"));
}

#[test]
fn test_sql_custom_view() {
    let dir = setup_custom_project(&[(
        "views.sql",
        r#"
CREATE VIEW active_users AS
SELECT id, name, email
FROM users
WHERE active = true;

CREATE VIEW product_sales AS
SELECT
    p.name,
    SUM(o.quantity) as total_sold,
    SUM(o.quantity * p.price) as revenue
FROM products p
JOIN orders o ON p.id = o.product_id
GROUP BY p.id, p.name;
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:active_users | def:product_sales")
        .assert()
        .success()
        .stdout(predicate::str::contains("views.sql"));
}

#[test]
fn test_sql_custom_cte() {
    let dir = setup_custom_project(&[(
        "queries.sql",
        r#"
WITH monthly_sales AS (
    SELECT
        DATE_TRUNC('month', order_date) as month,
        SUM(total) as sales
    FROM orders
    GROUP BY DATE_TRUNC('month', order_date)
),
avg_sales AS (
    SELECT AVG(sales) as average
    FROM monthly_sales
)
SELECT
    month,
    sales,
    sales - average as difference
FROM monthly_sales, avg_sales
ORDER BY month;
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("import:orders & ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("queries.sql"));
}

#[test]
fn test_sql_custom_constraints() {
    let dir = setup_custom_project(&[(
        "constraints.sql",
        r#"
CREATE TABLE employees (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    department_id INT REFERENCES departments(id),
    salary DECIMAL(10, 2) CHECK (salary > 0),
    hire_date DATE DEFAULT CURRENT_DATE
);

ALTER TABLE employees
ADD CONSTRAINT chk_email_format
CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$');
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:employees & ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains("constraints.sql"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_sql_format_paths() {
    let dir = setup_fixture("sql_generic");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("def:users")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("schema.sql"));
}

#[test]
fn test_sql_format_markdown() {
    let dir = setup_fixture("sql_generic");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("def:users")
        .assert()
        .success()
        .stdout(predicate::str::contains("```sql"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_sql_not_found() {
    let dir = setup_fixture("sql_generic");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:nonexistent_table & ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_sql_ext_filter() {
    let dir = setup_fixture("sql_generic");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:. & ext:sql")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
