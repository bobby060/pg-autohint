SELECT primaryname FROM title_principals
JOIN name_basics ON name_basics.nconst=title_principals.nconst
WHERE name_basics.primaryname='Ekta Kapoor' 
AND name_basics.birthyear=1975 
AND ((title_principals.category='producer'
AND title_principals.job='producer')
OR (title_principals.category='writer'
AND title_principals.job='developed by')
)
AND title_principals.characters IS NULL
AND title_principals.tconst LIKE 'tt%'
AND title_principals.ordering > 4
AND title_principals.ordering < 17;