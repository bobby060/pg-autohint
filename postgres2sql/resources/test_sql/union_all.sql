(SELECT primarytitle, startyear 
FROM title_basics 
WHERE startyear < 1950
LIMIT 100)
UNION 

(SELECT primarytitle, startyear
FROM title_basics
WHERE startyear >= 1950 AND startyear < 2000
LIMIT 100)

UNION  

(SELECT primarytitle, startyear
FROM title_basics
WHERE startyear >= 2000
LIMIT 100);