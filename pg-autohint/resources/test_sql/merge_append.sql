(SELECT * FROM title_ratings WHERE num_votes > 500000 AND average_rating < 4.0 ORDER BY num_votes DESC)
UNION ALL
(SELECT * FROM title_ratings WHERE num_votes < 10000 AND average_rating > 8.0 ORDER BY num_votes DESC)
ORDER BY num_votes DESC;