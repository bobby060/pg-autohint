SELECT *
FROM title_crew
WHERE tconst IN ('tt0000001', 'tt0000002', 'tt0100000')
INTERSECT ALL
SELECT *
FROM title_crew
WHERE tconst IN ('tt0000011', 'tt0100001', 'tt0100000');
