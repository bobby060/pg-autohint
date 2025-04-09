use crate::connector::convert_sql_file_to_plan;

pub mod connector;
pub mod hints;
pub mod optimize;
pub mod plan2ast;
pub mod postgresplan;
pub mod rule;
pub mod rules;

#[cfg(test)]
pub mod test_utils;

fn main() {
    let help = "Usage: postgres2sql convert [--analyze] <sql_file> <json_out_path>";

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 || args.len() > 5 {
        println!("{}", help);
        return;
    }

    let sql_file = if args.len() == 5 {
        args[3].clone()
    } else {
        args[2].clone()
    };
    let json_out_path = if args.len() == 5 {
        args[4].clone()
    } else {
        args[3].clone()
    };
    let analyze = if args.len() == 5 { true } else { false };

    convert_sql_file_to_plan(&sql_file, &json_out_path, analyze);
}
