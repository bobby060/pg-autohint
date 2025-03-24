
select primarytitle, startyear
from title_basics
where startyear > (
    select avg(startyear) 
    from title_basics
    where tconst in (
        select tconst
        from title_principals
        where category = 'actor'
        and nconst in (
            select nconst 
            from name_basics
            where primaryname like 'Tom%'
        )
    )
);
