extern crate tarpc;

use std::time::Instant;
use std::io::BufRead;

use analytics_lib::query::Query;
use client::{start_client, solution};

// Your solution goes here.
fn parse_query_from_string(input: String) -> Query {
    use analytics_lib::dataset::Value;
    use analytics_lib::query::{Aggregation, Condition, Query};

    fn strip_outer_parens(s: &str) -> &str {
        let s = s.trim();
        if !(s.starts_with('(') && s.ends_with(')')) {
            return s;
        }

        let mut depth = 0;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 && i != s.len() - 1 {
                        return s;
                    }
                }
                _ => {}
            }
        }

        &s[1..s.len() - 1]
    }

    fn find_top_level_op(s: &str, op: &str) -> Option<usize> {
        let bytes = s.as_bytes();
        let op_bytes = op.as_bytes();
        let mut depth = 0;
        let mut i = 0;

        while i + op_bytes.len() <= bytes.len() {
            match bytes[i] as char {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }

            if depth == 0 && &bytes[i..i + op_bytes.len()] == op_bytes {
                return Some(i);
            }

            i += 1;
        }

        None
    }

    fn parse_value(s: &str) -> Value {
        let s = s.trim();

        if s.starts_with('"') && s.ends_with('"') {
            return Value::String(s[1..s.len() - 1].to_string());
        }

        if let Ok(i) = s.parse::<i32>() {
            return Value::Integer(i);
        }

        panic!("Could not parse value: {}", s);
    }

    fn parse_condition(s: &str) -> Condition {
        let s = strip_outer_parens(s.trim());

        if s.starts_with("!(") && s.ends_with(')') {
            let inner = &s[2..s.len() - 1];
            return Condition::Not(Box::new(parse_condition(inner)));
        }

        if let Some(idx) = find_top_level_op(s, " OR ") {
            let left = &s[..idx];
            let right = &s[idx + 4..];
            return Condition::Or(
                Box::new(parse_condition(left)),
                Box::new(parse_condition(right)),
            );
        }

        if let Some(idx) = find_top_level_op(s, " AND ") {
            let left = &s[..idx];
            let right = &s[idx + 5..];
            return Condition::And(
                Box::new(parse_condition(left)),
                Box::new(parse_condition(right)),
            );
        }

        if let Some(idx) = s.find("==") {
            let column = s[..idx].trim().to_string();
            let value_str = s[idx + 2..].trim();
            let value = parse_value(value_str);
            return Condition::Equal(column, value);
        }

        panic!("Could not parse condition: {}", s);
    }

    let input = input.trim();

    let group_by_idx = input
        .find(" GROUP BY ")
        .expect("Query must contain `GROUP BY`");

    let filter_part = &input[..group_by_idx];
    let rest_part = &input[group_by_idx + " GROUP BY ".len()..];

    let filter_str = filter_part
        .strip_prefix("FILTER ")
        .expect("Query must start with `FILTER `");

    let filter = parse_condition(filter_str);

    let parts: Vec<&str> = rest_part.split_whitespace().collect();
    if parts.len() != 3 {
        panic!("Expected: GROUP BY <column> <AGG> <column>");
    }

    let group_by = parts[0].to_string();
    let agg_kind = parts[1];
    let agg_column = parts[2].to_string();

    let aggregate = match agg_kind {
        "COUNT" => Aggregation::Count(agg_column),
        "SUM" => Aggregation::Sum(agg_column),
        "AVERAGE" => Aggregation::Average(agg_column),
        _ => panic!("Unknown aggregation: {}", agg_kind),
    };

    Query::new(filter, group_by, aggregate)
}

// Each defined rpc generates an async fn that serves the RPC
#[tokio::main]
async fn main() {
    // Establish connection to server.
    let rpc_client = start_client().await;

    // Get a handle to the standard input stream
    let stdin = std::io::stdin();

    // Lock the handle to gain access to BufRead methods like lines()
    println!("Enter your query:");
    for line_result in stdin.lock().lines() {
        // Handle potential errors when reading a line
        match line_result {
            Ok(query) => {
                if query == "exit" {
                    break;
                }

                // parse query.
                let query = parse_query_from_string(query);

                // Carry out query.
                let time = Instant::now();
                let dataset = solution::run_fast_rpc(&rpc_client, query).await;
                let duration = time.elapsed();

                // Print results.
                println!("{}", dataset);
                println!("Query took {:?} to executed", duration);
                println!("Enter your next query (or enter exit to stop):");
            },
            Err(error) => {
                eprintln!("Error reading line: {}", error);
                break;
            }
        }
    }
}