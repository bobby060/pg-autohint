WITH top_movies AS (
    SELECT 
        tconst,
        average_rating
    FROM title_ratings
    WHERE average_rating > 8
)
SELECT 
    tb.startyear,
    tb.primarytitle,
    tm.average_rating
FROM top_movies tm INNER JOIN title_basics tb ON tm.tconst = tb.tconst
WHERE tb.startyear >= 2024
ORDER BY tb.startyear DESC, tm.average_rating DESC
LIMIT 20;
