-- postgres dialect marker
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    amount NUMERIC
);

CREATE OR REPLACE VIEW open_orders AS
SELECT amount FROM orders WHERE amount > 0;

CREATE OR REPLACE FUNCTION calculate_total(price NUMERIC, tax NUMERIC)
RETURNS TABLE(total NUMERIC)
LANGUAGE plpgsql
AS $$
BEGIN
  RETURN QUERY SELECT price + tax;
END;
$$;

SELECT calculate_total(10, 2);
SELECT amount FROM open_orders;
