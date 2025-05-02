use pgautohint::connector::*;
use pgautohint::hintengine::*;
use postgres::Client;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <query_path>", args[0]);
        std::process::exit(1);
    }

    let query_path = &args[1];
    println!("Query path: {}", query_path);

    let mut conn = establish_connection("imdbload", "postgres", "postgres", "localhost", "5432");

    // clear hint table before run
    let mut opt = Optimizer::new();
    opt.init_hint_table(&mut conn);
    println!("{}", "initialized hint table");

    // timeout queries to be excluded
    let excluded_queries = [
        "22c", "22d", "24a", "25a", "25c", "26a", "29c", "30c", "31c",
    ];

    // if is a directory, optimize all queries in dir
    let query_files = std::fs::read_dir(query_path).expect("Failed to read query directory");
    for entry in query_files {
        // use new optimizer to clear the states between queries
        let mut optimizer = Optimizer::new();
        // optimizer.add_rule(Box::new(rules::NljToHashJoin::new(1.0, 1500)));
        optimizer.add_rule(Box::new(rules::CardCorrection::new()));

        let entry = entry.expect("Failed to read directory entry");
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("sql") {
            // exclude timeout queries
            let path = entry.path();
            let file_name = path.file_stem().and_then(|stem| stem.to_str());
            if let Some(name) = file_name {
                if excluded_queries.contains(&name) {
                    println!("Skipping excluded query: {:?}", entry.path());
                    continue;
                }
            }
            // execute
            let query = std::fs::read_to_string(entry.path()).expect("Failed to read SQL file");
            println!("Optimizing query from file: {:?}", entry.path());
            match run_optimize(&mut conn, &mut optimizer, &query) {
                Ok(result) => {
                    if result.get_hints().is_none()
                        || result.get_hints().is_some_and(|x| x.size() == 0)
                    {
                        eprintln!(
                            "Warning query {:?}: produced no hints",
                            entry.path().to_str()
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Error optimizing query {:?}: {}", entry.path().to_str(), e);
                }
            }
        }
    }
}

fn run_optimize(
    conn: &mut Client,
    optimizer: &mut Optimizer,
    query: &str,
) -> Result<pgautohint::model::query::Query, String> {
    optimizer.optimize(query, conn, true, true, None)
}
