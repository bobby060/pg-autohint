-- sum 1 to 100, reference: https://www.postgresql.org/docs/current/queries-with.html#QUERIES-WITH-RECURSIVE
WITH RECURSIVE t(n) AS (
    VALUES (1)
  UNION ALL
    SELECT n+1 FROM t WHERE n < 100
)
SELECT sum(n) FROM t;