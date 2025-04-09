SELECT tconst, originaltitle, genres
FROM title_basics
WHERE tconst < 'tt0000009'
ORDER BY tconst
LIMIT 5
OFFSET 2;