# postgres-rel2sql




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

### Testing

Install code coverage tool `cargo +stable install cargo-llvm-cov --locked`

Redo code coverage test"
```cargo llvm-cov --html
```

## System design
1. Parse Postgres JSON into tree
2. Convert tree into SQL AST
3. Convert SQL AST into plain text



