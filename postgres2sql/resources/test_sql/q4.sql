select title_basics.primarytitle, name_basics.primaryname
from title_basics
join title_principals on title_basics.tconst = title_principals.tconst 
join name_basics on title_principals.nconst = name_basics.nconst;
