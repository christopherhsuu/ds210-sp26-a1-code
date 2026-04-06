use analytics_lib::dataset::{Dataset, Row, Value, ColumnType};
use analytics_lib::query::{Query, Condition, Aggregation};

pub fn hello() -> String {
    println!("hello called");
    return String::from("hello");
}

pub fn slow_rpc(input_dataset: &Dataset) -> Dataset {
    println!("slow_rpc called");
    let dataset = input_dataset.clone();
    return dataset
}

pub fn fast_rpc(input_dataset: &Dataset, query: Query) -> Dataset {
    println!("fast_rpc called");

    
    let condition = query.get_filter();
    let filtered_rows: Vec<&Row> = input_dataset.iter()
        .filter(|row| evaluate_condition(row, condition, input_dataset))
        .collect();

    
    let group_by_col = query.get_group_by();
    let group_by_index = input_dataset.column_index(group_by_col);
    let mut groups: std::collections::HashMap<Value, Vec<&Row>> = std::collections::HashMap::new();
    for row in filtered_rows {
        let key = row.get_value(group_by_index).clone();
        groups.entry(key).or_insert_with(Vec::new).push(row);
    }

    
    let aggregation = query.get_aggregate();
    let agg_col_name = aggregation.get_result_column_name();
    let mut result = Dataset::new(vec![
        (group_by_col.clone(), input_dataset.column_type(group_by_col).clone()),
        (agg_col_name, ColumnType::Integer),
    ]);

    for (group_value, rows) in groups {
        let agg_value = apply_aggregation(&rows, aggregation, input_dataset);
        result.add_row(Row::new(vec![group_value, Value::Integer(agg_value)]));
    }

    return result;
}

fn evaluate_condition(row: &Row, condition: &Condition, dataset: &Dataset) -> bool {
    match condition {
        Condition::Equal(col_name, value) => {
            let index = dataset.column_index(col_name);
            row.get_value(index) == value
        }
        Condition::Not(inner) => !evaluate_condition(row, inner, dataset),
        Condition::And(left, right) => {
            evaluate_condition(row, left, dataset) && evaluate_condition(row, right, dataset)
        }
        Condition::Or(left, right) => {
            evaluate_condition(row, left, dataset) || evaluate_condition(row, right, dataset)
        }
    }
}

fn apply_aggregation(rows: &[&Row], aggregation: &Aggregation, dataset: &Dataset) -> i32 {
    match aggregation {
        Aggregation::Count(_) => rows.len() as i32,
        Aggregation::Sum(col_name) => {
            let index = dataset.column_index(col_name);
            rows.iter().map(|row| match row.get_value(index) {
                Value::Integer(v) => *v,
                _ => panic!("Sum requires an Integer column"),
            }).sum()
        }
        Aggregation::Average(col_name) => {
            let index = dataset.column_index(col_name);
            let sum: i32 = rows.iter().map(|row| match row.get_value(index) {
                Value::Integer(v) => *v,
                _ => panic!("Average requires an Integer column"),
            }).sum();
            sum / rows.len() as i32
        }
    }
}