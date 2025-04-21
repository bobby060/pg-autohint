# original from https://gist.github.com/IllusiveMilkman/2a7a6614193c74804db7650f6d3c2bd2

printf "Script starting at %s. \n" "$(date)"

# printf "Removing old folders \n"
# rm -rf imdb-datasets/

# printf "Creating new folders \n"
# mkdir imdb-datasets/

# printf "Downloading datasets from https://datasets.imdbws.com \n"
# cd imdb-datasets
# curl -O https://datasets.imdbws.com/name.basics.tsv.gz
# curl -O https://datasets.imdbws.com/title.akas.tsv.gz
# curl -O https://datasets.imdbws.com/title.basics.tsv.gz
# curl -O https://datasets.imdbws.com/title.crew.tsv.gz
# curl -O https://datasets.imdbws.com/title.episode.tsv.gz
# curl -O https://datasets.imdbws.com/title.principals.tsv.gz
# curl -O https://datasets.imdbws.com/title.ratings.tsv.gz

# printf "Unzipping datasets... \n"
# gzip -dk *.gz
# cd ..

printf "Creating Database \n"
psql -U postgres -d 'postgres' -c "DROP DATABASE IF EXISTS imdb;"
psql -U postgres -d 'postgres' -c "CREATE DATABASE imdb;"

printf "Creating tables in imdb database \n"
psql -U postgres -d imdb -c "CREATE table title_ratings (tconst VARCHAR(10), average_rating NUMERIC,num_votes integer);"
psql -U postgres -d imdb -c "CREATE TABLE name_basics (nconst varchar(10) , primaryName text, birthYear smallint, deathYear smallint, primaryProfession text, knownForTitles text );"
psql -U postgres -d imdb -c "CREATE TABLE title_akas (titleId TEXT  , ordering INTEGER, title TEXT, region TEXT, language TEXT, types TEXT, attributes TEXT, isOriginalTitle BOOLEAN);"
psql -U postgres -d imdb -c "CREATE TABLE title_basics (tconst TEXT , titleType TEXT, primaryTitle TEXT, originalTitle TEXT, isAdult BOOLEAN, startYear SMALLINT, endYear SMALLINT, runtimeMinutes INTEGER, genres TEXT);"
psql -U postgres -d imdb -c "CREATE TABLE title_crew (tconst TEXT , directors TEXT, writers TEXT);"
psql -U postgres -d imdb -c "CREATE TABLE title_episode (const TEXT, parentTconst TEXT, seasonNumber TEXT, episodeNumber TEXT);"
psql -U postgres -d imdb -c "CREATE TABLE title_principals (tconst TEXT , ordering INTEGER, nconst TEXT, category TEXT, job TEXT, characters TEXT);"

printf "Inserting data into tables \n"
psql -U postgres -d imdb -c "\copy title_ratings FROM '$(pwd)/imdb-datasets/title.ratings.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -U postgres -d imdb -c "\copy name_basics FROM '$(pwd)/imdb-datasets/name.basics.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -U postgres -d imdb -c "\copy title_akas FROM '$(pwd)/imdb-datasets/title.akas.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -U postgres -d imdb -c "\copy title_basics FROM '$(pwd)/imdb-datasets/title.basics.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER" 
psql -U postgres -d imdb -c "\copy title_crew FROM '$(pwd)/imdb-datasets/title.crew.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -U postgres -d imdb -c "\copy title_episode FROM '$(pwd)/imdb-datasets/title.episode.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -U postgres -d imdb -c "\copy title_principals FROM '$(pwd)/imdb-datasets/title.principals.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
# Indexes on the primary keys of title_basics, title_ratings, title_crew, name_basics
printf "Creating indexes \n"
psql -U postgres -d imdb -c "CREATE INDEX title_basics_pkey ON title_basics (tconst);"
psql -U postgres -d imdb -c "CREATE INDEX title_ratings_pkey ON title_ratings (tconst);"
psql -U postgres -d imdb -c "CREATE INDEX title_crew_pkey ON title_crew (tconst);"
psql -U postgres -d imdb -c "CREATE INDEX name_basics_pkey ON name_basics (nconst);"
psql -U postgres -d imdb -c "CREATE INDEX num_votes_idx ON title_ratings (num_votes);"
<<<<<<< HEAD
# psql -U postgres -d imdb -c "CREATE INDEX episode_idx ON title_episode (parenttconst);"
# psql -U postgres -d imdb -c "CREATE INDEX episode_pkey ON title_episode (const);"


# Set up test case for index selection rule example
printf "Creating test case for index selection rule example \n"

# https://pganalyze.com/blog/5mins-postgres-planner-order-by-limit
# 
psql -U postgres -d imdb -c "CREATE TABLE orders_test(order_id int not null, shipping_date date not null,    PRIMARY KEY (order_id));"
psql -U postgres -d imdb -c "INSERT INTO orders_test SELECT generate_series(1, 2000000), '2018-01-01'::timestamp + random() * ('2022-05-01'::timestamp - '2018-01-01'::timestamp);"
psql -U postgres -d imdb -c "CREATE INDEX ON orders_test(shipping_date, order_id);"
psql -U postgres -d imdb -c "INSERT INTO orders_test SELECT generate_series(2000001, 2100000), '2022-05-01';"
psql -U postgres -d imdb -c "ANALYZE orders_test;"




=======
psql -U postgres -d imdb -c "CREATE INDEX average_rating_idx ON title_ratings (average_rating);"
>>>>>>> 14f37ab (1. Added new Postgres node types: BitmapHeapScan, BitmapIndexScan, SampleScan, WorkTableScan, CteScan, FunctionScan, BitmapOr, BitmapAnd, RecursiveUnion, ProjectSet.)
printf "Done! \n"

printf "Script done at %s. \n" "$(date)"
