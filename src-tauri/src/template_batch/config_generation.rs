use std::fs;

use calamine::{open_workbook_auto, Data, Reader};
use serde_json::{Map, Value};
use tera::Context;

use super::dto::{GenerateTemplateConfigsRequest, GenericGeneratedConfig};
use super::excel_utils::{
    columns_present, data_to_value, find_header_row, iterable_value_is_valid, row_is_empty,
    validate_columns,
};
use super::value_utils::trim_trailing_zeros;

#[tauri::command]
pub fn generate_template_configs(
    request: GenerateTemplateConfigsRequest,
) -> Result<Vec<GenericGeneratedConfig>, String> {
    if request.template_path.trim().is_empty() {
        return Err("请先上传 Tera 模板".into());
    }
    if request.excel_path.trim().is_empty() {
        return Err("请先选择 Excel 数据文件".into());
    }

    let template_content =
        fs::read_to_string(&request.template_path).map_err(|err| format!("读取模板失败: {err}"))?;

    let mut workbook = open_workbook_auto(&request.excel_path)
        .map_err(|err| format!("无法打开 Excel 文件: {err}"))?;
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Excel 文件中没有工作表".into());
    }

    let selected_sheet = request
        .sheet_name
        .filter(|name| sheet_names.contains(name))
        .unwrap_or_else(|| sheet_names[0].clone());

    let range = workbook
        .worksheet_range(&selected_sheet)
        .map_err(|err| format!("读取工作表失败: {err}"))?;

    let (header_row_index, columns) =
        find_header_row(&range).ok_or_else(|| "无法定位表头行".to_string())?;
    validate_columns(&columns, &request.expected_variables)?;

    let parsed_columns: Vec<(String, Vec<PathSegment>)> = columns
        .iter()
        .map(|name| (name.clone(), parse_path_segments(name)))
        .collect();

    let expected_segments: Vec<(String, Vec<PathSegment>)> = request
        .expected_variables
        .iter()
        .map(|name| (name.clone(), parse_path_segments(name)))
        .collect();

    let label_segments = request
        .label_field
        .as_ref()
        .map(|field| parse_path_segments(field));

    if let (Some(field), Some(_segments)) = (&request.label_field, &label_segments) {
        if !columns_present(&columns, field) {
            return Err(format!("Excel 表头中缺少标识列 {field}"));
        }
    }

    let mut results = Vec::new();

    for (row_index, row) in range.rows().enumerate() {
        if row_index <= header_row_index {
            continue;
        }
        if row_is_empty(row) {
            continue;
        }

        let mut root = Value::Object(Map::new());

        for (idx, (name, segments)) in parsed_columns.iter().enumerate() {
            if name.trim().is_empty() {
                continue;
            }
            let data = row.get(idx).unwrap_or(&Data::Empty);
            let cell_value = data_to_value(data);
            if matches!(cell_value, Value::Null) {
                ensure_path(&mut root, segments);
                continue;
            }
            insert_value(&mut root, segments, cell_value)?;
        }

        for (_, segments) in expected_segments.iter() {
            ensure_path(&mut root, segments);
        }

        ensure_iterable_columns_are_structured(&root, &parsed_columns, &request.iterable_variables)
            .map_err(|err| format!("第 {} 行: {err}", row_index + 1))?;

        let label = label_segments
            .as_ref()
            .and_then(|segments| extract_value(&root, segments))
            .and_then(value_to_display)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("行{}", row_index + 1));

        let context = Context::from_value(root).map_err(|err| format!("构建上下文失败: {err}"))?;
        let rendered = tera::Tera::one_off(&template_content, &context, false)
            .map_err(|err| format!("渲染模板失败: {err}"))?;

        results.push(GenericGeneratedConfig {
            label,
            config: rendered,
            row_index: row_index + 1,
        });
    }

    if results.is_empty() {
        return Err("未在 Excel 中找到有效数据行".into());
    }

    Ok(results)
}

fn column_segments<'a>(
    parsed_columns: &'a [(String, Vec<PathSegment>)],
    target: &str,
) -> Option<&'a [PathSegment]> {
    let normalized = target.trim();
    parsed_columns
        .iter()
        .find(|(name, _)| name.trim() == normalized)
        .map(|(_, segments)| segments.as_slice())
}

#[derive(Clone)]
enum PathSegment {
    Key(String),
    Index(usize),
}

fn parse_path_segments(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current.is_empty() {
                    segments.push(PathSegment::Key(current.clone()));
                    current.clear();
                }
            }
            '[' => {
                if !current.is_empty() {
                    segments.push(PathSegment::Key(current.clone()));
                    current.clear();
                }
                let mut content = String::new();
                for next in chars.by_ref() {
                    if next == ']' {
                        break;
                    }
                    content.push(next);
                }
                let content = content.trim();
                if content.starts_with('"') || content.starts_with('\'') {
                    let trimmed = content.trim_matches('"').trim_matches('\'');
                    segments.push(PathSegment::Key(trimmed.to_string()));
                } else if let Ok(index) = content.parse::<usize>() {
                    segments.push(PathSegment::Index(index));
                } else if !content.is_empty() {
                    segments.push(PathSegment::Key(content.to_string()));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        segments.push(PathSegment::Key(current));
    }
    segments
}

fn insert_value(target: &mut Value, path: &[PathSegment], value: Value) -> Result<(), String> {
    if path.is_empty() {
        return Err("无效的列路径".into());
    }
    match &path[0] {
        PathSegment::Key(key) => {
            if !target.is_object() {
                *target = Value::Object(Map::new());
            }
            let map = target.as_object_mut().unwrap();
            if path.len() == 1 {
                map.insert(key.clone(), value);
                return Ok(());
            }
            let entry = map.entry(key.clone()).or_insert(Value::Null);
            insert_value(entry, &path[1..], value)
        }
        PathSegment::Index(index) => {
            if !target.is_array() {
                *target = Value::Array(vec![]);
            }
            let array = target.as_array_mut().unwrap();
            if *index >= array.len() {
                array.resize(*index + 1, Value::Null);
            }
            if path.len() == 1 {
                array[*index] = value;
                return Ok(());
            }
            insert_value(&mut array[*index], &path[1..], value)
        }
    }
}

fn ensure_path(target: &mut Value, path: &[PathSegment]) {
    let mut current = target;
    for segment in path.iter() {
        match segment {
            PathSegment::Key(key) => {
                if !current.is_object() {
                    *current = Value::Object(Map::new());
                }
                let map = current.as_object_mut().unwrap();
                current = map.entry(key.clone()).or_insert(Value::Null);
            }
            PathSegment::Index(index) => {
                if !current.is_array() {
                    *current = Value::Array(vec![]);
                }
                let array = current.as_array_mut().unwrap();
                if *index >= array.len() {
                    array.resize(*index + 1, Value::Null);
                }
                current = &mut array[*index];
            }
        }
    }
}

fn ensure_iterable_columns_are_structured(
    root: &Value,
    parsed_columns: &[(String, Vec<PathSegment>)],
    iterable_variables: &[String],
) -> Result<(), String> {
    if iterable_variables.is_empty() {
        return Ok(());
    }
    let mut invalid = Vec::new();
    for variable in iterable_variables {
        if let Some(path) = column_segments(parsed_columns, variable) {
            if let Some(value) = extract_value(root, path) {
                if !iterable_value_is_valid(value) {
                    invalid.push(variable.clone());
                }
            } else {
                invalid.push(variable.clone());
            }
        }
    }
    if invalid.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "以下列必须填写由 JSON 对象组成的数组（或对象本身）: {}",
            invalid.join(", ")
        ))
    }
}

fn extract_value<'a>(value: &'a Value, path: &[PathSegment]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.iter() {
        match segment {
            PathSegment::Key(key) => current = current.get(key)?,
            PathSegment::Index(index) => current = current.get(*index)?,
        }
    }
    Some(current)
}

fn value_to_display(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(i.to_string())
            } else if let Some(u) = n.as_u64() {
                Some(u.to_string())
            } else if let Some(f) = n.as_f64() {
                let mut text = format!("{f}");
                if let Some(trimmed) = trim_trailing_zeros(&text) {
                    text = trimmed;
                }
                Some(text)
            } else {
                Some(n.to_string())
            }
        }
        Value::Bool(b) => Some(b.to_string()),
        other => Some(other.to_string()),
    }
}
