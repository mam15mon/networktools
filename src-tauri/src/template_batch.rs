use calamine::{open_workbook_auto, Data, Range, Reader};
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use tera::ast;
use tera::Context;
use tera::Template;

const MAX_PREVIEW_ROWS: usize = 100;
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeTeraTemplateRequest {
    pub file_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeraTemplateAnalysis {
    pub variables: Vec<String>,
    pub variable_count: usize,
    pub has_loops: bool,
    pub has_conditionals: bool,
    pub loop_count: usize,
    pub conditional_count: usize,
    #[serde(default)]
    pub iterable_variables: Vec<String>,
    #[serde(default)]
    pub sample_values: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub default_fallbacks: HashMap<String, String>,
    #[serde(default)]
    pub filter_usage: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTeraTemplateRequest {
    pub path: String,
    pub variables: Vec<String>,
    #[serde(default)]
    pub iterable_variables: Vec<String>,
    #[serde(default)]
    pub sample_values: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub default_fallbacks: HashMap<String, String>,
    #[serde(default)]
    pub filter_usage: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewTemplateExcelRequest {
    pub file_path: String,
    pub sheet_name: Option<String>,
    pub expected_variables: Vec<String>,
    #[serde(default)]
    pub iterable_variables: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateExcelPreview {
    pub sheet_names: Vec<String>,
    pub selected_sheet: String,
    pub header_row_index: usize,
    pub columns: Vec<String>,
    pub preview_rows: Vec<Vec<String>>,
    pub total_rows: usize,
    pub columns_with_data: Vec<String>,
    #[serde(default)]
    pub invalid_iterable_columns: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateConfigsRequest {
    pub template_path: String,
    pub excel_path: String,
    pub sheet_name: Option<String>,
    pub expected_variables: Vec<String>,
    pub label_field: Option<String>,
    #[serde(default)]
    pub iterable_variables: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenericGeneratedConfig {
    pub label: String,
    pub config: String,
    pub row_index: usize,
}

#[tauri::command]
pub fn analyze_tera_template(
    request: AnalyzeTeraTemplateRequest,
) -> Result<TeraTemplateAnalysis, String> {
    let content = fs::read_to_string(&request.file_path)
        .map_err(|err| format!("读取 Tera 模板失败: {err}"))?;

    let template = Template::new("__analysis__", None, &content)
        .map_err(|err| format_tera_parse_error(err, &content))?;
    let nodes = &template.ast;

    let mut result = AnalysisResult::default();
    let mut scope = Scope::default();
    collect_from_nodes(nodes, &mut scope, &mut result);

    let variables = result.variables;
    let variable_count = variables.len();
    let mut iterable_variables: Vec<String> = result.iterable_variables.into_iter().collect();
    iterable_variables.sort();
    let sample_values = result
        .sample_values
        .into_iter()
        .map(|(key, values)| {
            let mut list: Vec<String> = values.into_iter().collect();
            list.sort();
            (key, list)
        })
        .collect();
    let default_fallbacks = result.default_fallbacks.into_iter().collect();
    let filter_usage = result
        .filter_usage
        .into_iter()
        .map(|(key, set)| {
            let mut list: Vec<String> = set.into_iter().collect();
            list.sort();
            (key, list)
        })
        .collect();

    Ok(TeraTemplateAnalysis {
        variable_count,
        variables,
        has_loops: result.has_loops,
        has_conditionals: result.has_conditionals,
        loop_count: result.loop_count,
        conditional_count: result.conditional_count,
        iterable_variables,
        sample_values,
        default_fallbacks,
        filter_usage,
    })
}

#[tauri::command]
pub fn export_tera_variable_template(request: ExportTeraTemplateRequest) -> Result<(), String> {
    if request.variables.is_empty() {
        return Err("没有可导出的变量".into());
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let base_header_format = Format::new().set_bold().set_align(FormatAlign::Center);
    let base_sample_format = Format::new().set_align(FormatAlign::Left).set_text_wrap();

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

#[tauri::command]
pub fn preview_template_excel(
    request: PreviewTemplateExcelRequest,
) -> Result<TemplateExcelPreview, String> {
    let mut workbook = open_workbook_auto(&request.file_path)
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

    let column_count = columns.len();
    let (preview_rows, total_rows, column_non_empty_counts) =
        collect_preview_rows(&range, header_row_index, column_count);
    let columns_with_data = columns
        .iter()
        .enumerate()
        .filter_map(|(idx, name)| {
            if column_non_empty_counts
                .get(idx)
                .copied()
                .unwrap_or_default()
                > 0
            {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();

    let invalid_iterable_columns = request
        .iterable_variables
        .iter()
        .filter_map(|variable| {
            let idx = find_column_index(&columns, variable)?;
            match assess_iterable_column(&range, header_row_index, idx) {
                IterableColumnState::Invalid => Some(variable.clone()),
                _ => None,
            }
        })
        .collect();

    Ok(TemplateExcelPreview {
        sheet_names,
        selected_sheet,
        header_row_index,
        columns,
        preview_rows,
        total_rows,
        columns_with_data,
        invalid_iterable_columns,
    })
}

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

#[tauri::command]
pub fn export_template_configs(
    path: String,
    configs: Vec<GenericGeneratedConfig>,
) -> Result<(), String> {
    if configs.is_empty() {
        return Err("没有可导出的配置".into());
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (col, config) in configs.iter().enumerate() {
        worksheet
            .write(0u32, col as u16, &config.label)
            .map_err(|err| err.to_string())?;
        for (row_offset, line) in config.config.lines().enumerate() {
            worksheet
                .write((row_offset + 1) as u32, col as u16, line)
                .map_err(|err| err.to_string())?;
        }
    }

    workbook.save(path).map_err(|err| err.to_string())
}

#[derive(Default)]
struct AnalysisResult {
    variables: Vec<String>,
    seen: HashSet<String>,
    has_loops: bool,
    has_conditionals: bool,
    loop_count: usize,
    conditional_count: usize,
    iterable_variables: HashSet<String>,
    sample_values: HashMap<String, HashSet<String>>,
    default_fallbacks: HashMap<String, String>,
    filter_usage: HashMap<String, HashSet<String>>,
}

#[derive(Default, Clone)]
struct Scope {
    layers: Vec<HashSet<String>>,
}

impl Scope {
    fn with_locals<F>(&mut self, locals: &[String], mut f: F)
    where
        F: FnMut(&mut Scope),
    {
        let mut layer = HashSet::new();
        for name in locals {
            layer.insert(name.clone());
        }
        self.layers.push(layer);
        f(self);
        self.layers.pop();
    }

    fn insert_global(&mut self, name: String) {
        if self.layers.is_empty() {
            self.layers.push(HashSet::new());
        }
        if let Some(base) = self.layers.first_mut() {
            base.insert(name);
        }
    }

    fn is_local(&self, ident: &str) -> bool {
        let root = extract_ident_root(ident);
        self.layers.iter().rev().any(|layer| layer.contains(&root))
    }
}

fn collect_from_nodes(nodes: &[ast::Node], scope: &mut Scope, result: &mut AnalysisResult) {
    for node in nodes {
        match node {
            ast::Node::VariableBlock(_, expr) => collect_from_expr(expr, scope, result),
            ast::Node::If(if_block, _) => {
                result.has_conditionals = true;
                result.conditional_count += 1;
                for (_, expr, body) in &if_block.conditions {
                    collect_from_expr(expr, scope, result);
                    collect_condition_samples(expr, scope, result);
                    collect_from_nodes(body, scope, result);
                }
                if let Some((_, body)) = &if_block.otherwise {
                    collect_from_nodes(body, scope, result);
                }
            }
            ast::Node::Forloop(_, forloop, _) => {
                result.has_loops = true;
                result.loop_count += 1;
                collect_from_expr(&forloop.container, scope, result);
                if let Some(container) = extract_expr_ident(&forloop.container) {
                    result.iterable_variables.insert(container);
                }
                let mut locals = vec![forloop.value.clone(), "loop".to_string()];
                if let Some(key) = &forloop.key {
                    locals.push(key.clone());
                }
                scope.with_locals(&locals, |scope| {
                    collect_from_nodes(&forloop.body, scope, result);
                });
                if let Some(empty_body) = &forloop.empty_body {
                    collect_from_nodes(empty_body, scope, result);
                }
            }
            ast::Node::FilterSection(_, filter_section, _) => {
                collect_from_fn_call(&filter_section.filter, scope, result);
                collect_from_nodes(&filter_section.body, scope, result);
            }
            ast::Node::Block(_, block, _) => {
                collect_from_nodes(&block.body, scope, result);
            }
            ast::Node::MacroDefinition(_, definition, _) => {
                let arg_names: Vec<String> = definition.args.keys().cloned().collect();
                for default in definition.args.values().filter_map(|expr| expr.as_ref()) {
                    collect_from_expr(default, scope, result);
                }
                scope.with_locals(&arg_names, |scope| {
                    collect_from_nodes(&definition.body, scope, result);
                });
            }
            ast::Node::Set(_, set) => {
                collect_from_expr(&set.value, scope, result);
                scope.insert_global(set.key.clone());
            }
            ast::Node::Raw(_, _, _)
            | ast::Node::Text(_)
            | ast::Node::Comment(_, _)
            | ast::Node::Extends(_, _)
            | ast::Node::Include(_, _, _)
            | ast::Node::ImportMacro(_, _, _)
            | ast::Node::Super
            | ast::Node::Break(_)
            | ast::Node::Continue(_) => {}
        }
    }
}

fn collect_from_expr(expr: &ast::Expr, scope: &Scope, result: &mut AnalysisResult) {
    match &expr.val {
        ast::ExprVal::Ident(ident) => record_identifier(ident, scope, result),
        ast::ExprVal::Math(math) => {
            collect_from_expr(&math.lhs, scope, result);
            collect_from_expr(&math.rhs, scope, result);
        }
        ast::ExprVal::Logic(logic) => {
            collect_from_expr(&logic.lhs, scope, result);
            collect_from_expr(&logic.rhs, scope, result);
        }
        ast::ExprVal::Array(values) => {
            for value in values {
                collect_from_expr(value, scope, result);
            }
        }
        ast::ExprVal::FunctionCall(call) => {
            collect_from_fn_call(call, scope, result);
        }
        ast::ExprVal::MacroCall(call) => {
            for expr in call.args.values() {
                collect_from_expr(expr, scope, result);
            }
        }
        ast::ExprVal::StringConcat(concat) => {
            for value in &concat.values {
                if let ast::ExprVal::Ident(ident) = value {
                    record_identifier(ident, scope, result);
                }
            }
        }
        ast::ExprVal::Test(test) => {
            record_identifier(&test.ident, scope, result);
            for arg in &test.args {
                collect_from_expr(arg, scope, result);
            }
        }
        ast::ExprVal::In(op) => {
            collect_from_expr(&op.lhs, scope, result);
            collect_from_expr(&op.rhs, scope, result);
        }
        ast::ExprVal::String(_)
        | ast::ExprVal::Int(_)
        | ast::ExprVal::Float(_)
        | ast::ExprVal::Bool(_) => {}
    }

    for filter in &expr.filters {
        collect_from_fn_call(filter, scope, result);
    }

    record_filter_metadata(expr, scope, result);
}

fn collect_from_fn_call(call: &ast::FunctionCall, scope: &Scope, result: &mut AnalysisResult) {
    for expr in call.args.values() {
        collect_from_expr(expr, scope, result);
    }
}

fn record_filter_metadata(expr: &ast::Expr, scope: &Scope, result: &mut AnalysisResult) {
    let ident = match extract_full_ident(expr) {
        Some(ident) => ident,
        None => return,
    };
    if scope.is_local(&ident) {
        return;
    }
    if expr.filters.is_empty() {
        return;
    }
    for filter in &expr.filters {
		result
			.filter_usage
			.entry(ident.clone())
			.or_default()
			.insert(filter.name.clone());
        if filter.name == "default" {
            if let Some(description) = describe_default_filter(filter) {
                result.default_fallbacks.insert(ident.clone(), description);
            }
        }
    }
}

fn collect_condition_samples(expr: &ast::Expr, scope: &Scope, result: &mut AnalysisResult) {
    match &expr.val {
        ast::ExprVal::Logic(logic) => match logic.operator {
            ast::LogicOperator::And | ast::LogicOperator::Or => {
                collect_condition_samples(&logic.lhs, scope, result);
                collect_condition_samples(&logic.rhs, scope, result);
            }
            ast::LogicOperator::Eq | ast::LogicOperator::NotEq => {
                record_logic_sample(&logic.lhs, &logic.rhs, scope, result);
                record_logic_sample(&logic.rhs, &logic.lhs, scope, result);
            }
            _ => {
                collect_condition_samples(&logic.lhs, scope, result);
                collect_condition_samples(&logic.rhs, scope, result);
            }
        },
        ast::ExprVal::In(in_expr) => {
            if let Some(ident) = extract_full_ident(&in_expr.lhs) {
                if scope.is_local(&ident) {
                    return;
                }
                if let Some(values) = extract_array_literals(&in_expr.rhs) {
                    for value in values {
                        record_sample(&ident, value, result);
                    }
                }
            }
        }
        _ => {}
    }
}

fn extract_expr_ident(expr: &ast::Expr) -> Option<String> {
    if let ast::ExprVal::Ident(ident) = &expr.val {
        Some(extract_ident_root(ident))
    } else {
        None
    }
}

fn record_identifier(ident: &str, scope: &Scope, result: &mut AnalysisResult) {
    if ident.is_empty() || scope.is_local(ident) {
        return;
    }
    if result.seen.insert(ident.to_string()) {
        result.variables.push(ident.to_string());
    }
}

fn record_sample(ident: &str, value: String, result: &mut AnalysisResult) {
	result
		.sample_values
		.entry(ident.to_string())
		.or_default()
		.insert(value);
}

fn extract_ident_root(ident: &str) -> String {
	ident
		.split(['.', '['])
		.next()
		.unwrap_or(ident)
		.to_string()
}

fn extract_full_ident(expr: &ast::Expr) -> Option<String> {
    if expr.negated {
        return None;
    }
    if let ast::ExprVal::Ident(ident) = &expr.val {
        Some(ident.clone())
    } else {
        None
    }
}

fn record_logic_sample(
    lhs: &ast::Expr,
    rhs: &ast::Expr,
    scope: &Scope,
    result: &mut AnalysisResult,
) {
    let ident = match extract_full_ident(lhs) {
        Some(ident) => ident,
        None => return,
    };
    if scope.is_local(&ident) {
        return;
    }
    if let Some(value) = literal_expr_to_string(rhs) {
        record_sample(&ident, value, result);
    }
}

fn literal_expr_to_string(expr: &ast::Expr) -> Option<String> {
    if expr.negated {
        return None;
    }
    match &expr.val {
        ast::ExprVal::String(value) => Some(value.clone()),
        ast::ExprVal::Int(value) => Some(value.to_string()),
        ast::ExprVal::Float(value) => {
            let text = value.to_string();
            Some(trim_trailing_zeros(&text).unwrap_or(text))
        }
        ast::ExprVal::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn extract_array_literals(expr: &ast::Expr) -> Option<Vec<String>> {
    if expr.negated {
        return None;
    }
    if let ast::ExprVal::Array(values) = &expr.val {
        let mut literals = Vec::new();
        for value in values {
            if let Some(text) = literal_expr_to_string(value) {
                literals.push(text);
            } else {
                return None;
            }
        }
        Some(literals)
    } else {
        None
    }
}

fn describe_default_filter(call: &ast::FunctionCall) -> Option<String> {
    if call.args.is_empty() {
        return None;
    }
    if let Some(expr) = call.args.get("value") {
        return describe_default_argument(expr);
    }
    if let Some(expr) = call.args.get("_0") {
        return describe_default_argument(expr);
    }
    if let Some((_, expr)) = call.args.iter().next() {
        return describe_default_argument(expr);
    }
    None
}

fn describe_default_argument(expr: &ast::Expr) -> Option<String> {
    if let Some(ident) = extract_full_ident(expr) {
        return Some(ident);
    }
    literal_expr_to_string(expr)
}

fn find_header_row(range: &Range<Data>) -> Option<(usize, Vec<String>)> {
    range.rows().enumerate().find_map(|(idx, row)| {
        let columns = row.iter().map(data_type_to_string).collect::<Vec<_>>();
        if columns.iter().all(|col| col.trim().is_empty()) {
            None
        } else {
            Some((idx, columns))
        }
    })
}

fn validate_columns(columns: &[String], required: &[String]) -> Result<(), String> {
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

fn columns_present(columns: &[String], target: &str) -> bool {
    columns.iter().any(|column| column.trim() == target.trim())
}

fn find_column_index(columns: &[String], target: &str) -> Option<usize> {
    let normalized = target.trim();
    columns
        .iter()
        .position(|column| column.trim() == normalized)
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

fn is_iterable_variable(name: &str, iterable_variables: &[String]) -> bool {
    let normalized = name.trim();
    iterable_variables
        .iter()
        .any(|item| item.trim() == normalized)
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
) -> String {
    if classification.is_iterable {
        return format!(
            "[{{\"id\":1,\"name\":\"{}示例\",\"ip\":\"10.0.0.1\",\"mask\":\"255.255.255.0\"}}]",
            name
        );
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

fn collect_preview_rows(
    range: &Range<Data>,
    header_row_index: usize,
    column_count: usize,
) -> (Vec<Vec<String>>, usize, Vec<usize>) {
    let mut preview_rows = Vec::new();
    let mut total_rows = 0usize;
    let mut column_non_empty_counts = vec![0usize; column_count];

    for (idx, row) in range.rows().enumerate() {
        if idx <= header_row_index {
            continue;
        }
        if row_is_empty(row) {
            continue;
        }
        total_rows += 1;

        if preview_rows.len() < MAX_PREVIEW_ROWS {
            let mut formatted_row = Vec::with_capacity(column_count);
            for col_idx in 0..column_count {
		let value = row
			.get(col_idx)
			.map(data_type_to_string)
			.unwrap_or_default();
                if !value.is_empty() {
                    if let Some(count) = column_non_empty_counts.get_mut(col_idx) {
                        *count += 1;
                    }
                }
                formatted_row.push(value);
            }
            preview_rows.push(formatted_row);
        } else {
            for col_idx in 0..column_count {
                if let Some(cell) = row.get(col_idx) {
                    if data_has_value(cell) {
                        if let Some(count) = column_non_empty_counts.get_mut(col_idx) {
                            *count += 1;
                        }
                    }
                }
            }
        }
    }

    (preview_rows, total_rows, column_non_empty_counts)
}

enum IterableColumnState {
    Empty,
    Valid,
    Invalid,
}

fn assess_iterable_column(
    range: &Range<Data>,
    header_row_index: usize,
    column_index: usize,
) -> IterableColumnState {
    let mut seen_valid = false;
    for (idx, row) in range.rows().enumerate() {
        if idx <= header_row_index {
            continue;
        }
        if let Some(cell) = row.get(column_index) {
            if !data_has_value(cell) {
                continue;
            }
            let value = data_to_value(cell);
            if iterable_value_is_valid(&value) {
                seen_valid = true;
                continue;
            } else {
                return IterableColumnState::Invalid;
            }
        }
    }
    if seen_valid {
        IterableColumnState::Valid
    } else {
        IterableColumnState::Empty
    }
}

fn row_is_empty(row: &[Data]) -> bool {
    row.iter().all(|cell| data_type_to_string(cell).is_empty())
}

fn data_type_to_string(value: &Data) -> String {
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

fn data_has_value(value: &Data) -> bool {
    match value {
        Data::String(s) => !s.trim().is_empty(),
        Data::Float(_) | Data::Int(_) | Data::Bool(_) => true,
        Data::Empty | Data::Error(_) => false,
        other => !other.to_string().trim().is_empty(),
    }
}

fn data_to_value(value: &Data) -> Value {
    match value {
        Data::String(s) => parse_string_value(s),
        Data::Float(v) => float_to_number(*v),
        Data::Int(v) => Value::Number((*v).into()),
        Data::Bool(v) => Value::Bool(*v),
        Data::Empty | Data::Error(_) => Value::Null,
        other => Value::String(other.to_string()),
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

fn iterable_value_is_valid(value: &Value) -> bool {
    match value {
        Value::Array(items) => items.iter().all(|item| item.is_object()),
        Value::Object(_) => true,
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

fn trim_trailing_zeros(input: &str) -> Option<String> {
    if let Some(point_pos) = input.find('.') {
        let mut end = input.len();
        while end > point_pos + 1 && input.as_bytes()[end - 1] == b'0' {
            end -= 1;
        }
        if end > point_pos + 1 && input.as_bytes()[end - 1] == b'.' {
            end -= 1;
        }
        if end != input.len() {
            return Some(input[..end].to_string());
        }
    }
    None
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

fn format_tera_parse_error(err: tera::Error, template: &str) -> String {
    let raw = err.to_string();
    if let Some((line, column)) = extract_line_column(&raw) {
        let raw_line = template.lines().nth(line.saturating_sub(1)).unwrap_or("");
        if raw_line.is_empty() {
            return format!("模板语法错误（第 {line} 行，第 {column} 列）: {raw}");
        }

        let (pointer_line, highlighted_char) = build_pointer_line(raw_line, column);
        let mut message = format!(
            "模板语法错误（第 {line} 行，第 {column} 列）:\n{raw_line}\n{pointer_line}\n详情: {raw}"
        );
        if let Some(ch) = highlighted_char {
            if let Some(hint) = describe_character_issue(raw_line, ch, line, column) {
                message.push_str("\n提示: ");
                message.push_str(&hint);
            }
        }
        message
    } else {
        format!("模板语法错误: {raw}")
    }
}

fn extract_line_column(raw: &str) -> Option<(usize, usize)> {
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("-->") {
            let rest = rest.trim();
            let (line_part, tail) = rest.split_once(':')?;
            let line_num = line_part.trim().parse().ok()?;
            let column_part = tail
                .trim()
				.split(['|', ' '])
                .find(|chunk| !chunk.is_empty())?;
            let column_num = column_part.trim().parse().ok()?;
            return Some((line_num, column_num));
        }
    }
    None
}

fn build_pointer_line(raw_line: &str, column: usize) -> (String, Option<char>) {
    let mut pointer = String::new();
    let mut highlighted_char = None;
    let mut char_count = 0;
    for ch in raw_line.chars() {
        char_count += 1;
        if char_count == column {
            pointer.push('^');
            highlighted_char = Some(ch);
        } else {
            pointer.push(if ch == '\t' { '\t' } else { ' ' });
        }
    }
    if column > char_count {
        pointer.push_str(&" ".repeat(column - char_count - 1));
        pointer.push('^');
    }
    (pointer, highlighted_char)
}

fn describe_character_issue(
    raw_line: &str,
    ch: char,
    line: usize,
    column: usize,
) -> Option<String> {
    if !ch.is_ascii() {
        return Some(format!(
            "第 {line} 行第 {column} 列字符 `{ch}` 为非 ASCII，可能导致变量或语法无法解析。"
        ));
    }

    if ch.is_ascii_alphanumeric() || ch == '_' {
        if let Some(hint) = detect_identifier_with_space(raw_line, line, column) {
            return Some(hint);
        }
        return None;
    }

    match ch {
        '{' => Some(format!(
            "第 {line} 行第 {column} 列为 `{{`，请检查是否正确使用 `{{{{` 和 `}}}}` 或 `{{%` 和 `%}}`。"
        )),
        '}' => Some(format!(
            "第 {line} 行第 {column} 列为 `}}`，请检查是否缺少对应的 `{{` 或 `%` 块闭合。"
        )),
        '%' => Some(format!(
            "第 {line} 行第 {column} 列为 `%`，请确认是否使用 `{{% ... %}}` 包围控制语句。"
        )),
        '#' => Some(format!(
            "第 {line} 行第 {column} 列为 `#`，请确认 `{{#` 与 `#}}` 注释是否成对出现。"
        )),
        '|' => Some(format!(
            "第 {line} 行第 {column} 列为 `|`，请确认过滤器语法是否正确，例如 `{{ value | upper }}`。"
        )),
        '.' => Some(format!(
            "第 {line} 行第 {column} 列为 `.`，请确保前面是对象变量，后面是有效的属性名。"
        )),
        '[' => Some(format!(
            "第 {line} 行第 {column} 列为 `[`，请确保索引访问有对应的 `]` 闭合。"
        )),
        ']' => Some(format!(
            "第 {line} 行第 {column} 列为 `]`，请检查是否缺少对应的 `[`。"
        )),
        '(' => Some(format!(
            "第 {line} 行第 {column} 列为 `(`，请确保函数调用或分组语法完整，并有 `)` 闭合。"
        )),
        ')' => Some(format!(
            "第 {line} 行第 {column} 列为 `)`，请检查是否缺少对应的 `(`。"
        )),
        '+' => Some(format!(
            "第 {line} 行第 {column} 列为 `+`，请确认两侧为可参与运算的数值。"
        )),
        '*' => Some(format!(
            "第 {line} 行第 {column} 列为 `*`，请确认两侧为可参与运算的数值。"
        )),
        '/' => Some(format!(
            "第 {line} 行第 {column} 列为 `/`，请确认两侧为可参与运算的数值且分母不为 0。"
        )),
        '=' => Some(format!(
            "第 {line} 行第 {column} 列为 `=`，可能是比较或赋值运算符一部分，请检查语法。"
        )),
        '!' => Some(format!(
            "第 {line} 行第 {column} 列为 `!`，可能是逻辑非或比较运算符一部分，请检查语法。"
        )),
        '>' => Some(format!(
            "第 {line} 行第 {column} 列为 `>`，比较运算符，请确认两侧可比较。"
        )),
        '<' => Some(format!(
            "第 {line} 行第 {column} 列为 `<`，比较运算符，请确认两侧可比较。"
        )),
        ':' => Some(format!(
            "第 {line} 行第 {column} 列为 `:`，常用于 for 循环或过滤器参数分隔，请检查上下文。"
        )),
        ';' => Some(format!(
            "第 {line} 行第 {column} 列为 `;`，Tera 模板很少使用，请确认是否必要。"
        )),
        '"' => Some(format!(
            "第 {line} 行第 {column} 列为 `\"`，请确保字符串有对应的结束引号。"
        )),
        '\'' => Some(format!(
            "第 {line} 行第 {column} 列为 `'`，请确保字符串有对应的结束引号。"
        )),
        '@' => Some(format!(
            "第 {line} 行第 {column} 列为 `@`，可能用于宏调用或路径，请检查语法。"
        )),
        '-' => Some(format!(
            "第 {line} 行第 {column} 列包含 `-`，变量名不支持该字符，请改用下划线或拆分表达式。"
        )),
        ' ' => Some(format!(
            "第 {line} 行第 {column} 列为空格，请确保变量标识符与运算符之间的语法正确。"
        )),
        '\t' => Some(format!(
            "第 {line} 行第 {column} 列为制表符，请确认没有多余缩进影响语法。"
        )),
        _ => Some(format!(
            "第 {line} 行第 {column} 列字符 `{ch}` 在模板中可能有特殊含义，请检查是否缺少过滤器、括号或闭合标记。"
        )),
    }
}

fn detect_identifier_with_space(raw_line: &str, line: usize, column: usize) -> Option<String> {
    let chars: Vec<char> = raw_line.chars().collect();
    if column == 0 || column > chars.len() {
        return None;
    }
    let idx = column - 1;
    if !is_identifier_char(chars[idx]) {
        return None;
    }
    if idx == 0 || !chars[idx - 1].is_whitespace() {
        return None;
    }
    let prev_non_ws = chars[..idx].iter().rposition(|c| !c.is_whitespace())?;
    if !is_identifier_char(chars[prev_non_ws]) {
        return None;
    }

    let mut start_first = prev_non_ws;
    while start_first > 0 && is_identifier_char(chars[start_first - 1]) {
        start_first -= 1;
    }
    let first = chars[start_first..=prev_non_ws].iter().collect::<String>();

    let mut end_second = idx;
    while end_second + 1 < chars.len() && is_identifier_char(chars[end_second + 1]) {
        end_second += 1;
    }
    let second = chars[idx..=end_second].iter().collect::<String>();
    if first.is_empty() || second.is_empty() {
        return None;
    }

    Some(format!(
        "第 {line} 行变量名 `{first}` 与 `{second}` 中间存在空格，请移除空格或使用下划线连接，例如 `{first}{second}`。",
        first = first,
        second = second
    ))
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
