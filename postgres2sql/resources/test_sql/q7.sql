
SELECT primarytitle, startyear
FROM title_basics tb
JOIN title_ratings r ON tb.tconst = r.tconst
JOIN title_crew tc ON tb.tconst = tc.tconst
WHERE (startyear > 2000 )
    OR (r.num_votes > 1000000)
    AND NOT tc.directors LIKE '%Tom%'
LIMIT 5;
