select primaryname 
from name_basics, title_principals, title_basics
where name_basics.nconst = title_principals.nconst
and title_basics.tconst = title_principals.tconst
limit 10;
