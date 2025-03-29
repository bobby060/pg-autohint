SELECT 
    tb.tconst,
    tb.primarytitle,
    tb.titletype,
    tb.startyear,
    tr.average_rating
FROM title_basics tb
LEFT OUTER JOIN title_ratings tr ON tb.tconst = tr.tconst
WHERE tr.tconst IS NULL
LIMIT 10;
