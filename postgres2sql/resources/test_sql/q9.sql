select primarytitle, runtimeminutes
from title_basics 
where runtimeminutes < 30
union
select primarytitle, runtimeminutes
from title_basics
where runtimeminutes > 180
