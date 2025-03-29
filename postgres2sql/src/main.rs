// use postgres2sql;
use postgres2sql::connector::convert_sql_file_to_plan;

fn main() {
    let help = "Usage: postgres2sql convert <sql_file> <json_out_path>";

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        println!("{:?}", args);
        println!("{}", help);
        return;
    }

    let sql_file = args[2].clone();
    let json_out_path = args[3].clone();

    convert_sql_file_to_plan(&sql_file, &json_out_path);
}
