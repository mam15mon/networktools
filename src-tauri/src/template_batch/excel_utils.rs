use calamine::{Data, Range};
use serde_json::{Number, Value};

pub(super) fn find_header_row(range: &Range<Data>) -> Option<(usize, Vec<String>)> {
    range.rows().enumerate().find_map(|(idx, row)| {
        let columns = row.iter().map(data_type_to_string).collect::<Vec<_>>();
        if columns.iter().all(|col| col.trim().is_empty()) {
            None
        } else {
            Some((idx, columns))
        }
    })
}

pub(super) fn validate_columns(columns: &[String], required: &[String]) -> Result<(), String> {
    if required.is_empty() {
        return Ok(());
    }
    let missing: Vec<String> = required
        .iter()
        .filter(|name| !columns_present(columns, name))
        .cloned()
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("Excel 表头缺少以下列: {}", missing.join(", ")))
    }
}

pub(super) fn columns_present(columns: &[String], target: &str) -> bool {
    columns.iter().any(|column| column.trim() == target.trim())
}

pub(super) fn find_column_index(columns: &[String], target: &str) -> Option<usize> {
    let normalized = target.trim();
    columns
        .iter()
        .position(|column| column.trim() == normalized)
}

pub(super) fn row_is_empty(row: &[Data]) -> bool {
    row.iter().all(|cell| data_type_to_string(cell).is_empty())
}

pub(super) fn data_type_to_string(value: &Data) -> String {
    match value {
        Data::String(s) => s.trim().to_string(),
        Data::Float(v) => {
            if (v.fract()).abs() < f64::EPSILON {
                format!("{:.0}", v)
            } else {
                v.to_string()
            }
        }
        Data::Int(v) => v.to_string(),
        Data::Bool(v) => v.to_string(),
        Data::Empty | Data::Error(_) => String::new(),
        other => other.to_string(),
    }
}

pub(super) fn data_has_value(value: &Data) -> bool {
    match value {
        Data::String(s) => !s.trim().is_empty(),
        Data::Float(_) | Data::Int(_) | Data::Bool(_) => true,
        Data::Empty | Data::Error(_) => false,
        other => !other.to_string().trim().is_empty(),
    }
}

pub(super) fn data_to_value(value: &Data) -> Value {
    match value {
        Data::String(s) => parse_string_value(s),
        Data::Float(v) => float_to_number(*v),
        Data::Int(v) => Value::Number((*v).into()),
        Data::Bool(v) => Value::Bool(*v),
        Data::Empty | Data::Error(_) => Value::Null,
        other => Value::String(other.to_string()),
    }
}

pub(super) fn iterable_value_is_valid(value: &Value) -> bool {
    match value {
        Value::Array(items) => items.iter().all(|item| !item.is_array()),
        Value::Object(_) => true,
        Value::Bool(_) | Value::Number(_) | Value::String(_) => true,
        _ => false,
    }
}

fn parse_string_value(raw: &str) -> Value {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Value::Null
    } else if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
        parsed
    } else {
        Value::String(trimmed.to_string())
    }
}

fn float_to_number(value: f64) -> Value {
    if value.is_finite() {
        let truncated = value.trunc();
        if (value - truncated).abs() < f64::EPSILON
            && truncated >= i64::MIN as f64
            && truncated <= i64::MAX as f64
        {
            return Value::Number((truncated as i64).into());
        }
    }
    Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}
