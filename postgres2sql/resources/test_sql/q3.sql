select primaryname 
from name_basics, title_principals, title_basics
where name_basics.nconst = title_principals.nconst
and title_principals.category = 'actor'
and title_principals.job = 'actor'
and title_basics.tconst = title_principals.tconst
and title_basics.genres like '%Comedy%'
