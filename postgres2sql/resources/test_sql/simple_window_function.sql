SELECT *
FROM (
    SELECT 
        tb.genres,
        tb.primarytitle,
        tr.average_rating,
        RANK() OVER (PARTITION BY tb.genres ORDER BY tr.average_rating DESC) AS rank_in_genre
    FROM title_basics tb
    JOIN title_ratings tr ON tb.tconst = tr.tconst
    WHERE tb.titletype = 'movie' AND tb.genres IS NOT NULL
) ranked
WHERE rank_in_genre <= 3
ORDER BY genres, rank_in_genre
LIMIT 10;