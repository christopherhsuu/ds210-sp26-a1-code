use std::collections::HashMap;
use crate::dataset::{ColumnType, Dataset, Value, Row};
use crate::query::{Aggregation, Condition, Query};

pub fn filter_dataset(dataset: &Dataset, filter: &Condition) -> Dataset {
    let mut result = Dataset::new(dataset.columns().clone());

    for row in dataset.iter() {
        if matches_condition(dataset, row, filter) {
            result.add_row(row.clone());
        }
    }

    result
}

fn matches_condition(dataset: &Dataset, row: &Row, condition: &Condition) -> bool {
    match condition {
        Condition::Equal(column_name, value) => {
            let index = dataset.column_index(column_name);
            row.get_value(index) == value
        }
        Condition::Not(inner) => {
            !matches_condition(dataset, row, inner)
        }
        Condition::And(left, right) => {
            matches_condition(dataset, row, left) && matches_condition(dataset, row, right)
        }
        Condition::Or(left, right) => {
            matches_condition(dataset, row, left) || matches_condition(dataset, row, right)
        }
    }
}


pub fn group_by_dataset(dataset: Dataset, group_by_column: &String) -> HashMap<Value, Dataset> {
    let mut groups: HashMap<Value, Dataset> = HashMap::new();
    let col_index = dataset.column_index(group_by_column);
    let columns = dataset.columns().clone();

    for row in dataset.into_iter() {
        let key = row.get_value(col_index).clone();
        groups
            .entry(key)
            .or_insert_with(|| Dataset::new(columns.clone()))
            .add_row(row);
    }

    groups
}

pub fn aggregate_dataset(dataset: HashMap<Value, Dataset>, aggregation: &Aggregation) -> HashMap<Value, Value> {
      let mut result = HashMap::new();

    for (group_value, group_dataset) in dataset {
        let aggregated_value = match aggregation {
            Aggregation::Count(_column_name) => {
                Value::Integer(group_dataset.len() as i32)
            }

            Aggregation::Sum(column_name) => {
                let column_index = group_dataset.column_index(column_name);
                let mut sum = 0;

                for row in group_dataset.iter() {
                    if let Value::Integer(value) = row.get_value(column_index) {
                        sum += *value;
                    }
                }

                Value::Integer(sum)
            }

            Aggregation::Average(column_name) => {
                let column_index = group_dataset.column_index(column_name);
                let mut sum = 0;
                let mut count = 0;

                for row in group_dataset.iter() {
                    if let Value::Integer(value) = row.get_value(column_index) {
                        sum += *value;
                        count += 1;
                    }
                }

                Value::Integer(sum / count)
            }
        };

        result.insert(group_value, aggregated_value);
    }

    result
}


pub fn compute_query_on_dataset(dataset: &Dataset, query: &Query) -> Dataset {
    let filtered = filter_dataset(dataset, query.get_filter());
    let grouped = group_by_dataset(filtered, query.get_group_by());
    let aggregated = aggregate_dataset(grouped, query.get_aggregate());

    // Create the name of the columns.
    let group_by_column_name = query.get_group_by();
    let group_by_column_type = dataset.column_type(group_by_column_name);
    let columns = vec![
        (group_by_column_name.clone(), group_by_column_type.clone()),
        (query.get_aggregate().get_result_column_name(), ColumnType::Integer),
    ];

    // Create result dataset object and fill it with the results.
    let mut result = Dataset::new(columns);
    for (grouped_value, aggregation_value) in aggregated {
        result.add_row(Row::new(vec![grouped_value, aggregation_value]));
    }
    return result;
}