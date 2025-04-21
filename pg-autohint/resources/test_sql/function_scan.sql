SELECT unnested_genre, COUNT(*) as movie_count
FROM (
    SELECT unnest(string_to_array(genres, ',')) AS unnested_genre
    FROM title_basics
    WHERE titletype = 'movie' AND startyear in (SELECT * FROM generate_series(2010, 2020))
) t
GROUP BY unnested_genre
ORDER BY movie_count DESC;