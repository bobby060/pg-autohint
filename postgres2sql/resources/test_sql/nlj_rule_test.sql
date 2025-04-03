SELECT title_principals.tconst FROM title_principals
JOIN name_basics ON name_basics.nconst=title_principals.nconst
JOIN title_basics ON title_principals.tconst=title_basics.tconst
WHERE 
title_basics.isadult='f'
AND (name_basics.primaryname='Ekta Kapoor' 
OR name_basics.primaryname='Shobha Kapoor'
OR name_basics.primaryname='Snehasish Chakraborty')
AND (name_basics.birthyear=1975 
OR name_basics.birthyear IS NULL)
AND ((title_principals.category='producer'
AND title_principals.job='producer')
OR (title_principals.category='writer'
AND title_principals.job='developed by')
)
AND title_principals.characters IS NULL
AND title_principals.tconst LIKE 'tt%'
AND title_principals.ordering > 6
AND title_principals.ordering < 17;