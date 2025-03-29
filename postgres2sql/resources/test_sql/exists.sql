SELECT tb.tconst, tb.primarytitle, tb.titletype
FROM title_basics tb
WHERE NOT EXISTS (
    SELECT 1
    FROM title_akas ta
    WHERE ta.titleid = tb.tconst
)
LIMIT 10;
