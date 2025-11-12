use std::collections::HashMap;

use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};

use super::dto::ExportTeraTemplateRequest;

const CONDITIONAL_COLOR: Color = Color::RGB(0xD6EAF8);
const DEFAULT_COLOR: Color = Color::RGB(0xFCF3CF);
const FORMATTING_COLOR: Color = Color::RGB(0xE8DAEF);
const FORMATTING_FILTERS: &[&str] = &[
    "upper",
    "lower",
    "capitalize",
    "title",
    "trim",
    "trim_end",
    "trim_start",
    "slice",
    "replace",
    "escape",
];

#[tauri::command]
pub fn export_tera_variable_template(request: ExportTeraTemplateRequest) -> Result<(), String> {
    if request.variables.is_empty() {
        return Err("没有可导出的变量".into());
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let base_header_format = Format::new().set_bold().set_align(FormatAlign::Center);
    let base_sample_format = Format::new().set_align(FormatAlign::Left).set_text_wrap();
    let fallback_consumers = build_fallback_consumers(&request.default_fallbacks);

    for (col, variable) in request.variables.iter().enumerate() {
        let classification = classify_column(variable, &request);
        let header_format = build_colored_format(&base_header_format, &classification);
        worksheet
            .write_with_format(0, col as u16, variable, &header_format)
            .map_err(|err| err.to_string())?;
        let sample_value = compose_sample_value(
            variable,
            &classification,
            &request.sample_values,
            &request.default_fallbacks,
            &request.iterable_fields,
            &fallback_consumers,
        );
        let sample_format = build_colored_format(&base_sample_format, &classification);
        worksheet
            .write_with_format(1, col as u16, &sample_value, &sample_format)
            .map_err(|err| err.to_string())?;
    }

    write_color_legend(&mut workbook).map_err(|err| err.to_string())?;

    workbook
        .save(request.path)
        .map_err(|err| format!("保存模板失败: {err}"))
}

struct ColumnClassification {
    is_iterable: bool,
    conditional: bool,
    defaultable: bool,
    formatting: bool,
}

fn classify_column(name: &str, request: &ExportTeraTemplateRequest) -> ColumnClassification {
    ColumnClassification {
        is_iterable: is_iterable_variable(name, &request.iterable_variables),
        conditional: request
            .sample_values
            .get(name)
            .map(|values| !values.is_empty())
            .unwrap_or(false),
        defaultable: request.default_fallbacks.contains_key(name),
        formatting: is_pure_formatting(name, &request.filter_usage),
    }
}

fn is_iterable_variable(name: &str, iterable_variables: &[String]) -> bool {
    let normalized = name.trim();
    iterable_variables
        .iter()
        .any(|item| item.trim() == normalized)
}

fn is_pure_formatting(name: &str, filter_usage: &HashMap<String, Vec<String>>) -> bool {
    let filters = match filter_usage.get(name) {
        Some(list) => list,
        None => return false,
    };
    if filters.is_empty() {
        return false;
    }
    filters
        .iter()
        .all(|filter| FORMATTING_FILTERS.contains(&filter.as_str()))
}

fn build_colored_format(base: &Format, classification: &ColumnClassification) -> Format {
    if let Some(color) = column_color(classification) {
        base.clone().set_background_color(color)
    } else {
        base.clone()
    }
}

fn column_color(classification: &ColumnClassification) -> Option<Color> {
    if classification.defaultable {
        Some(DEFAULT_COLOR)
    } else if classification.conditional {
        Some(CONDITIONAL_COLOR)
    } else if classification.formatting {
        Some(FORMATTING_COLOR)
    } else {
        None
    }
}

fn compose_sample_value(
    name: &str,
    classification: &ColumnClassification,
    sample_values: &HashMap<String, Vec<String>>,
    default_fallbacks: &HashMap<String, String>,
    iterable_fields: &HashMap<String, Vec<String>>,
    fallback_consumers: &HashMap<String, Vec<String>>,
) -> String {
    if classification.is_iterable {
        return compose_iterable_sample(name, iterable_fields.get(name));
    }

    let samples = sample_values.get(name);
    let default_hint = default_fallbacks.get(name);

    if let Some(default_hint) = default_hint {
        if let Some(values) = samples {
            if !values.is_empty() {
                return format!("可选：{}；默认 {}", format_examples(values), default_hint);
            }
        }
        return format!("默认 {}，可留空", default_hint);
    }

    if let Some(values) = samples {
        if !values.is_empty() {
            return format_examples(values);
        }
    }

    if classification.formatting {
        return format!("{} 示例值（自动格式化）", name);
    }

    if let Some(consumers) = fallback_consumers.get(name) {
        let listed = preview_consumers(consumers);
        return format!("供 {} 默认引用，请填写实际值", listed);
    }
    if let Some(example) = infer_numeric_example(name) {
        return example.to_string();
    }

    format!("{} 示例值", name)
}

fn write_color_legend(workbook: &mut Workbook) -> Result<(), rust_xlsxwriter::XlsxError> {
    let legend_sheet = workbook.add_worksheet();
    legend_sheet.set_name("颜色说明")?;

    let title_format = Format::new().set_bold();
    legend_sheet.write_with_format(0, 0, "颜色说明", &title_format)?;

    let conditional_format = Format::new()
        .set_background_color(CONDITIONAL_COLOR)
        .set_align(FormatAlign::Left);
    legend_sheet.write_with_format(
        1,
        0,
        "浅蓝：参与 if/elif 条件判断的变量",
        &conditional_format,
    )?;

    let default_format = Format::new()
        .set_background_color(DEFAULT_COLOR)
        .set_align(FormatAlign::Left);
    legend_sheet.write_with_format(2, 0, "浅黄：支持 default 回退，可留空", &default_format)?;

    let formatting_format = Format::new()
        .set_background_color(FORMATTING_COLOR)
        .set_align(FormatAlign::Left);
    legend_sheet.write_with_format(
        3,
        0,
        "浅紫：仅包含格式化过滤器（upper/capitalize 等）",
        &formatting_format,
    )?;

    Ok(())
}

fn format_examples(values: &[String]) -> String {
    if values.is_empty() {
        return String::new();
    }
    if values.len() == 1 {
        return values[0].clone();
    }
    let parts: Vec<String> = values.iter().take(3).cloned().collect();
    let preview = parts.join(" / ");
    if values.len() > 3 {
        format!("{preview} / ...")
    } else {
        preview
    }
}

fn compose_iterable_sample(name: &str, child_fields: Option<&Vec<String>>) -> String {
    let mut child_fields = child_fields.cloned().unwrap_or_default();
    if child_fields.is_empty() {
        if let Some(defaults) = infer_default_child_fields(name) {
            child_fields = defaults;
        }
    }

    if child_fields.is_empty() {
        return compose_scalar_iterable_sample(name);
    }

    let sample_count = 3;
    let mut entries = Vec::new();
    for index in 0..sample_count {
        let fields: Vec<String> = child_fields
            .iter()
            .map(|field| {
                let value = sample_iterable_field_value(field, name, index);
                format!("\"{field}\":\"{value}\"")
            })
            .collect();
        entries.push(format!("{{{}}}", fields.join(",")));
    }
    format!("[{}]", entries.join(","))
}

fn compose_scalar_iterable_sample(name: &str) -> String {
    if let Some(example) = infer_numeric_example(name) {
        if let Ok(base) = example.parse::<i64>() {
            return format!("[\"{}\",\"{}\",\"{}\"]", base, base + 1, base + 10);
        }
        return format!("[\"{example}\",\"{example}_2\",\"{example}_10\"]");
    }
    format!("[\"{name}1\",\"{name}2\",\"{name}10\"]")
}

fn sample_value_for_field(field: &str, parent: &str) -> String {
    if let Some(example) = infer_numeric_example(field) {
        return example.to_string();
    }
    if let Some(ip) = infer_ip_example(field) {
        return ip.to_string();
    }
    let normalized = normalized_name(field);
    if normalized.contains("name") {
        return format!("{parent}名称");
    }
    if normalized.contains("desc") || normalized.contains("remark") {
        return "示例描述".into();
    }
    format!("{field}示例")
}

fn sample_iterable_field_value(field: &str, parent: &str, index: usize) -> String {
    if let Some(example) = infer_numeric_example(field) {
        if let Ok(base) = example.parse::<i64>() {
            let offsets = [0, 1, 9];
            let offset = offsets.get(index).copied().unwrap_or(index as i64);
            return (base + offset).to_string();
        }
        return example.to_string();
    }
    if let Some(ip) = infer_ip_example(field) {
        let segments: Vec<&str> = ip.split('.').collect();
        if segments.len() == 4 {
            let mut octets: Vec<i32> = segments
                .iter()
                .filter_map(|segment| segment.parse::<i32>().ok())
                .collect();
            if octets.len() == 4 {
                octets[3] += index as i32;
                octets[3] = octets[3].max(1);
                return format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]);
            }
        }
        return ip.to_string();
    }
    let base = sample_value_for_field(field, parent);
    if index == 0 {
        base
    } else {
        format!("{base}_{}", index + 1)
    }
}

fn infer_numeric_example(name: &str) -> Option<&'static str> {
    let lowered = normalized_name(name).to_lowercase();
    if lowered.contains("vlan") {
        if lowered.contains("start") {
            return Some("100");
        }
        if lowered.contains("end") {
            return Some("200");
        }
        return Some("10");
    }
    if lowered.contains("start") {
        return Some("1");
    }
    if lowered.contains("end") {
        return Some("10");
    }
    if lowered.ends_with("_id") || lowered.ends_with("id") {
        return Some("1");
    }
    if lowered.ends_with("_number") || lowered.ends_with("_no") {
        return Some("42");
    }
    if lowered.ends_with("_count") || lowered.ends_with("_size") {
        return Some("2");
    }
    if lowered.contains("port") {
        return Some("48");
    }
    if lowered.contains("slot") {
        return Some("2");
    }
    None
}

fn infer_ip_example(name: &str) -> Option<&'static str> {
    let lowered = normalized_name(name).to_lowercase();
    if lowered.contains("gateway") {
        return Some("10.0.0.1");
    }
    if lowered.contains("loopback") {
        return Some("192.168.255.1");
    }
    if lowered.contains("mask") || lowered.contains("netmask") {
        return Some("255.255.255.0");
    }
    if lowered.contains("ip") {
        return Some("10.0.0.1");
    }
    None
}

fn infer_default_child_fields(name: &str) -> Option<Vec<String>> {
    let lowered = normalized_name(name).to_lowercase();
    if lowered.contains("vlan") {
        return Some(vec!["id".to_string()]);
    }
    if lowered.ends_with('s') {
        return Some(vec!["id".to_string()]);
    }
    None
}

fn normalized_name(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

fn build_fallback_consumers(
    default_fallbacks: &HashMap<String, String>,
) -> HashMap<String, Vec<String>> {
    let mut consumers: HashMap<String, Vec<String>> = HashMap::new();
    for (variable, fallback) in default_fallbacks {
        let entry = consumers.entry(fallback.clone()).or_default();
        entry.push(variable.clone());
    }
    consumers
}

fn preview_consumers(consumers: &[String]) -> String {
    match consumers.len() {
        0 => String::new(),
        1 => consumers[0].clone(),
        _ => {
            let mut listed = consumers.to_vec();
            listed.sort();
            if listed.len() > 3 {
                let preview = listed.iter().take(3).cloned().collect::<Vec<_>>();
                format!("{}, ...", preview.join(", "))
            } else {
                listed.join(", ")
            }
        }
    }
}
