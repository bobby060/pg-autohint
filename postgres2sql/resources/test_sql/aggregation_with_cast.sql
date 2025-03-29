SELECT 
    titletype,
    (CAST(startyear AS integer) / 10) * 10 AS decade,
    COUNT(*) AS title_count
FROM title_basics
WHERE startyear >= 1970
GROUP BY titletype, decade
ORDER BY decade, titletype;