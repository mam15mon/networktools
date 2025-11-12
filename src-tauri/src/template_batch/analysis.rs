use std::collections::{HashMap, HashSet};
use std::fs;

use tera::{ast, Template};

use super::dto::{AnalyzeTeraTemplateRequest, TeraTemplateAnalysis};
use super::value_utils::trim_trailing_zeros;

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
    let iterable_fields = result
        .iterable_field_usage
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
        iterable_fields,
        sample_values,
        default_fallbacks,
        filter_usage,
    })
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
    iterable_field_usage: HashMap<String, HashSet<String>>,
}

#[derive(Default, Clone)]
struct Scope {
    layers: Vec<HashSet<String>>,
    iterable_aliases: Vec<HashMap<String, String>>,
}

impl Scope {
    fn with_locals<F>(
        &mut self,
        locals: &[String],
        iterable_bindings: &[(String, String)],
        mut f: F,
    ) where
        F: FnMut(&mut Scope),
    {
        let mut layer = HashSet::new();
        for name in locals {
            layer.insert(name.clone());
        }
        self.layers.push(layer);
        let mut alias_map = HashMap::new();
        for (alias, parent) in iterable_bindings {
            alias_map.insert(alias.clone(), parent.clone());
        }
        self.iterable_aliases.push(alias_map);
        f(self);
        self.iterable_aliases.pop();
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

    fn iterable_parent_field(&self, ident: &str) -> Option<(String, String)> {
        let root = extract_ident_root(ident);
        let mut parts = ident.splitn(2, '.');
        parts.next()?;
        let field = parts.next()?.to_string();
        for map in self.iterable_aliases.iter().rev() {
            if let Some(parent) = map.get(&root) {
                return Some((parent.clone(), field));
            }
        }
        None
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
                let container_ident = extract_expr_ident(&forloop.container);
                if let Some(container) = &container_ident {
                    result.iterable_variables.insert(container.clone());
                }
                let mut locals = vec![forloop.value.clone(), "loop".to_string()];
                if let Some(key) = &forloop.key {
                    locals.push(key.clone());
                }
                let mut iterable_bindings = Vec::new();
                if let Some(container) = &container_ident {
                    iterable_bindings.push((forloop.value.clone(), container.clone()));
                }
                scope.with_locals(&locals, &iterable_bindings, |scope| {
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
                scope.with_locals(&arg_names, &[], |scope| {
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
            for arg in call.args.values() {
                collect_from_expr(arg, scope, result);
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
    if call.name == "range" {
        collect_from_range_call(call, scope, result);
        return;
    }
    for expr in call.args.values() {
        collect_from_expr(expr, scope, result);
    }
}

fn collect_from_range_call(call: &ast::FunctionCall, scope: &Scope, result: &mut AnalysisResult) {
    let mut visited = HashSet::new();
    if let Some(start_expr) = call.args.get("start").or_else(|| call.args.get("_0")) {
        collect_from_expr(start_expr, scope, result);
        visited.insert("start");
        visited.insert("_0");
    }
    if let Some(end_expr) = call.args.get("end").or_else(|| call.args.get("_1")) {
        collect_from_expr(end_expr, scope, result);
        visited.insert("end");
        visited.insert("_1");
    }
    for (key, expr) in call.args.iter() {
        if visited.contains(key.as_str()) {
            continue;
        }
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
    if ident.is_empty() {
        return;
    }
    if scope.is_local(ident) {
        if let Some((parent, field)) = scope.iterable_parent_field(ident) {
            if !field.is_empty() {
                result
                    .iterable_field_usage
                    .entry(parent)
                    .or_default()
                    .insert(field);
            }
        }
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
    ident.split(['.', '[']).next().unwrap_or(ident).to_string()
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
