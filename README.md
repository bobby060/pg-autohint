# pg-autohint




## Get started

1. Install rust

Linux/unix:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

2. Install postgres 16
```
sudo apt install postgresql

sudo systemctl restart postgresql.service

sudo -i -u postgres psql


```

Create a user with your machines username

```
sudo su - postgres
psql


```
Run `ALTER USER postgres PASSWORD 'postgres';` to set the password for the postgres user.

Run `CREATE ROLE <username> superuser createdb login;`, where username is your machines username.

Then copy the imdb_postgres_setup.sh to that user and run as postgres. I did this by touching a new file, then pasting the text, but there is probably a better way...


Now you can access imdb with `psql -d imdb` from your normal user

Run all tests
```cargo test```

Run specific test
```cargo test q10 -- --nocapture```

3. hint table
To enable hint table feature, connect to the database, load pg_hint_plan, create hint table, and enable hint table by
```psql
LOAD 'pg_hint_plan';
CREATE EXTENSION pg_hint_plan;
set pg_hint_plan.enable_hint_table='on';
```
To get query identifier, run
```psql
SET compute_query_id = 'on';
```
Then `EXPLAIN VERBOSE` returns query identifier, which would be used to insert hint into hint table

### Testing

Install code coverage tool `cargo +stable install cargo-llvm-cov --locked`

Redo code coverage test"
```cargo llvm-cov --html
```

### JOB benchmark
To load JOB benchmark
```
git clone https://github.com/danolivo/jo-bench
cd jo-bench

export PGDATABASE=imdbload
export PGPORT=5432
export PGHOST=localhost
export PGUSER=postgres
export PGPASSWORD=<your password>

psql -f schema.sql
cp -r ./csv /tmp/csv
psql -vdatadir="'/tmp'" -f ~/jo-bench/copy.sql
cd -
```

To run the original one pass
```
sudo vim /etc/postgresql/17/main/postgresql.conf
```
add
```
shared_preload_libraries = 'pg_stat_statements, pg_hint_plan'
```
then
```
sudo systemctl restart postgresql
cd job_benchmark/
./run_job.sh
cd -
```
it will generate `job-onepass-X.dat` and `explains-X.txt` that contains execution time for each query and its plan from `EXPLAIN (ANALYZE, VERBOSE, FORMAT JSON)`

To run hinted version
first generate hint table
```
cd job_benchmark/
cargo run --release <query dir>
cd -
```
Then run the hinted queries
```
cd job_benchmark/
./run_job_hinted.sh
cd -
```

## System design
1. Parse Postgres JSON into tree
2. Convert tree into SQL AST
3. Convert SQL AST into plain text
