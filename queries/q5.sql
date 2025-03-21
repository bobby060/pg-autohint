select primarytitle
from title_basics
where runtimeminutes > (
    select avg(runtimeminutes)
    from title_basics
    where runtimeminutes is not null
)
