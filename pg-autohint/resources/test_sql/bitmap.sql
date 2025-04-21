SELECT *
FROM title_ratings
WHERE num_votes = 290 OR num_votes = 3000
UNION
SELECT *
FROM title_ratings
WHERE average_rating < 4.0 AND num_votes > 5000;