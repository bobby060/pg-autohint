-- Find Directors with Highest Average Rating (for directors with at least 5 movies)
SELECT 
    nb.primaryname AS director_name,
    COUNT(tb.tconst) AS movie_count,
    AVG(tr.average_rating) AS avg_rating,
    MAX(tr.average_rating) AS max_rating,
    MIN(tr.average_rating) AS min_rating
FROM title_crew tc
JOIN name_basics nb ON tc.directors LIKE '%' || nb.nconst || '%'   -- string concatenation equivalent to '%nb.nconst%'
JOIN title_basics tb ON tc.tconst = tb.tconst
JOIN title_ratings tr ON tb.tconst = tr.tconst
WHERE tb.titletype = 'movie'
GROUP BY nb.primaryname
HAVING COUNT(tb.tconst) >= 5
ORDER BY avg_rating DESC
LIMIT 20;