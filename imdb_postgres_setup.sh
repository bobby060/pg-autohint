# original from https://gist.github.com/IllusiveMilkman/2a7a6614193c74804db7650f6d3c2bd2

# This script was intended to get raw IMDB datasets into Postgres
# The script worked on my mac setup.
#
# You may have to update your psql command with appropriate -U and -d flags and may have to 
# provide appropriate permissions to new folders.
#
# Customise as you see fit and for your setup.
#
# Tables are NOT optomised, nor have any indexes been created
# The point is to "just get the data into Postgres"
#
# Remember to allow execution rights before trying to run it:
# chmod 755 imdb_postgres_setup.sh
#
# Just for interest's sake, below are the terminal commands for my Linux box:
# su - postgres
# curl -O https://gist.githubusercontent.com/IllusiveMilkman/2a7a6614193c74804db7650f6d3c2bd2/raw/c8c0b4dbac00cf7539dd5cc9670fe00b38430f7d/imdb_postgres_setup.sh
# chmod 755 imdb_postgres_setup.sh
# ./imdb_postgres_setup.sh
#
# If you don't know the password for postgres (new install, some default VM setups, etc)
# sudo passwd postgres
# set a new password, then continue above
#

printf "Script starting at %s. \n" "$(date)"

printf "Removing old folders \n"
rm -rf imdb-datasets/

printf "Creating new folders \n"
mkdir imdb-datasets/

printf "Downloading datasets from https://datasets.imdbws.com \n"
cd imdb-datasets
curl -O https://datasets.imdbws.com/name.basics.tsv.gz
curl -O https://datasets.imdbws.com/title.akas.tsv.gz
curl -O https://datasets.imdbws.com/title.basics.tsv.gz
curl -O https://datasets.imdbws.com/title.crew.tsv.gz
curl -O https://datasets.imdbws.com/title.episode.tsv.gz
curl -O https://datasets.imdbws.com/title.principals.tsv.gz
curl -O https://datasets.imdbws.com/title.ratings.tsv.gz

printf "Unzipping datasets... \n"
gzip -dk *.gz
cd ..

printf "Creating Database \n"
psql -d 'postgres' -c "DROP DATABASE IF EXISTS imdb;"
psql -d 'postgres' -c "CREATE DATABASE imdb;"

printf "Creating tables in imdb database \n"
psql -d imdb -c "CREATE table title_ratings (tconst VARCHAR(10), average_rating NUMERIC,num_votes integer);"
psql -d imdb -c "CREATE TABLE name_basics (nconst varchar(10) , primaryName text, birthYear smallint, deathYear smallint, primaryProfession text, knownForTitles text );"
psql -d imdb -c "CREATE TABLE title_akas (titleId TEXT  , ordering INTEGER, title TEXT, region TEXT, language TEXT, types TEXT, attributes TEXT, isOriginalTitle BOOLEAN);"
psql -d imdb -c "CREATE TABLE title_basics (tconst TEXT , titleType TEXT, primaryTitle TEXT, originalTitle TEXT, isAdult BOOLEAN, startYear SMALLINT, endYear SMALLINT, runtimeMinutes INTEGER, genres TEXT);"
psql -d imdb -c "CREATE TABLE title_crew (tconst TEXT , directors TEXT, writers TEXT);"
psql -d imdb -c "CREATE TABLE title_episode (const TEXT, parentTconst TEXT, seasonNumber TEXT, episodeNumber TEXT);"
psql -d imdb -c "CREATE TABLE title_principals (tconst TEXT , ordering INTEGER, nconst TEXT, category TEXT, job TEXT, characters TEXT);"


printf "Inserting data into tables \n"
psql -d imdb -c "COPY title_ratings FROM '$(pwd)/imdb-datasets/title.ratings.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -d imdb -c "COPY name_basics FROM '$(pwd)/imdb-datasets/name.basics.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -d imdb -c "COPY title_akas FROM '$(pwd)/imdb-datasets/title.akas.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -d imdb -c "COPY title_basics FROM '$(pwd)/imdb-datasets/title.basics.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -d imdb -c "COPY title_crew FROM '$(pwd)/imdb-datasets/title.crew.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -d imdb -c "COPY title_episode FROM '$(pwd)/imdb-datasets/title.episode.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"
psql -d imdb -c "COPY title_principals FROM '$(pwd)/imdb-datasets/title.principals.tsv' DELIMITER E'\t' QUOTE E'\b' NULL '\N' CSV HEADER"


# Indexes on the primary keys of title_basics, title_ratings, title_crew, name_basics
printf "Creating indexes \n"
psql -d imdb -c "CREATE INDEX title_basics_pkey ON title_basics (tconst);"
psql -d imdb -c "CREATE INDEX title_ratings_pkey ON title_ratings (tconst);"
psql -d imdb -c "CREATE INDEX title_crew_pkey ON title_crew (tconst);"
psql -d imdb -c "CREATE INDEX name_basics_pkey ON name_basics (nconst);"

printf "Done! \n"

printf "Script done at %s. \n" "$(date)"