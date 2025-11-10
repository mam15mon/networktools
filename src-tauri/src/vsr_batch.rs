use calamine::{open_workbook_auto, Data, Range, Reader};
use ipnet::Ipv4Net;
use rust_xlsxwriter::{Format, FormatAlign, Workbook};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use strsim::normalized_levenshtein;
use tera::Context;

const MAX_PREVIEW_ROWS: usize = 100;
const VSR_TEMPLATE: &str = include_str!("../templates/vsr_config.tera");
const DERIVED_POOL_FIELDS: &[&str] = &["start_ip", "end_ip", "pool_ip_gateway"];

#[derive(Debug)]
struct PoolComputation {
    gateway: String,
    start: String,
    end: String,
}

const VSR_FIELDS: &[FieldMeta] = &[
    FieldMeta::new("device_name", "设备名称", true, FieldCategory::Core),
    FieldMeta::new("ip", "接入接口 IP", true, FieldCategory::Core),
    FieldMeta::new("gateway", "默认网关", true, FieldCategory::Core),
    FieldMeta::new("vsr_username", "管理员用户名", false, FieldCategory::Local),
    FieldMeta::new("vsr_password", "管理员密码", false, FieldCategory::Local),
    FieldMeta::new("monitor_username", "监控账号", false, FieldCategory::Local),
    FieldMeta::new("monitor_password", "监控密码", false, FieldCategory::Local),
    FieldMeta::new("ppp_username", "PPP 本地用户", false, FieldCategory::Local),
    FieldMeta::new("ppp_password", "PPP 密码", false, FieldCategory::Local),
    FieldMeta::new("pool_cidr", "地址池 CIDR", true, FieldCategory::Core),
    FieldMeta::new(
        "ldap_server_ip",
        "LDAP 服务器 IP",
        false,
        FieldCategory::Ldap,
    ),
    FieldMeta::new("ldap_login_dn", "LDAP Login DN", false, FieldCategory::Ldap),
    FieldMeta::new(
        "ldap_search_base_dn",
        "LDAP Search Base DN",
        false,
        FieldCategory::Ldap,
    ),
    FieldMeta::new("ldap_password", "LDAP 密码", false, FieldCategory::Ldap),
    FieldMeta::new(
        "radius_ip",
        "Radius 服务器 IP",
        false,
        FieldCategory::Radius,
    ),
    FieldMeta::new(
        "radius_password",
        "Radius 密码",
        false,
        FieldCategory::Radius,
    ),
];

const COLUMN_PATTERNS: &[(&str, &[&str])] = &[
    (
        "device_name",
        &["device", "device_name", "name", "设备", "设备名称", "主机"],
    ),
    ("ip", &["ip", "接口ip", "vsr_ip", "管理ip", "address"]),
    ("gateway", &["gateway", "gw", "默认网关", "出口", "下一跳"]),
    (
        "pool_cidr",
        &["pool_cidr", "cidr", "地址池", "地址池cidr", "pool"],
    ),
    (
        "vsr_username",
        &["vsr_username", "admin", "管理员", "vsr用户"],
    ),
    (
        "vsr_password",
        &["vsr_password", "admin_password", "管理员密码"],
    ),
    (
        "monitor_username",
        &["monitor_username", "monitor", "监控账号"],
    ),
    (
        "monitor_password",
        &["monitor_password", "monitor_pwd", "监控密码"],
    ),
    ("ppp_username", &["ppp_username", "ppp用户", "local_ppp"]),
    ("ppp_password", &["ppp_password", "ppp_pwd", "ppp密码"]),
    (
        "ldap_server_ip",
        &["ldap_server_ip", "ldap_ip", "ldap服务器"],
    ),
    (
        "ldap_login_dn",
        &["ldap_login_dn", "login_dn", "ldap login"],
    ),
    (
        "ldap_search_base_dn",
        &["ldap_search_base_dn", "search_dn", "search_base"],
    ),
    ("ldap_password", &["ldap_password", "ldap_pwd", "ldap密码"]),
    ("radius_ip", &["radius_ip", "radius", "radius服务器"]),
    (
        "radius_password",
        &["radius_password", "radius_pwd", "radius密码"],
    ),
];

#[derive(Clone, Copy)]
struct FieldMeta {
    key: &'static str,
    _label: &'static str,
    required: bool,
    category: FieldCategory,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldCategory {
    Core,
    Local,
    Ldap,
    Radius,
}

impl FieldMeta {
    const fn new(
        key: &'static str,
        label: &'static str,
        required: bool,
        category: FieldCategory,
    ) -> Self {
        FieldMeta {
            key,
            _label: label,
            required,
            category,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcelAnalysis {
    pub sheet_names: Vec<String>,
    pub selected_sheet: String,
    pub header_row_index: usize,
    pub columns: Vec<String>,
    pub preview_rows: Vec<Vec<String>>,
    pub suggested_mapping: HashMap<String, String>,
    pub total_rows: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessExcelRequest {
    pub file_path: String,
    pub sheet_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertVsrRequest {
    pub file_path: String,
    pub sheet_name: Option<String>,
    pub header_row_index: usize,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VsrDeviceEntry {
    pub device_name: String,
    pub ip: String,
    pub gateway: String,
    pub vsr_username: Option<String>,
    pub vsr_password: Option<String>,
    pub monitor_username: Option<String>,
    pub monitor_password: Option<String>,
    pub ppp_username: Option<String>,
    pub ppp_password: Option<String>,
    pub pool_cidr: Option<String>,
    pub start_ip: String,
    pub end_ip: String,
    pub pool_ip_gateway: String,
    pub ldap_server_ip: Option<String>,
    pub ldap_login_dn: Option<String>,
    pub ldap_search_base_dn: Option<String>,
    pub ldap_password: Option<String>,
    pub radius_ip: Option<String>,
    pub radius_password: Option<String>,
    pub row_index: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertResponse {
    pub entries: Vec<VsrDeviceEntry>,
    pub errors: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateVsrConfigRequest {
    pub entries: Vec<VsrDeviceEntry>,
    pub include_local: bool,
    pub include_ldap: bool,
    pub include_radius: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VsrGeneratedConfig {
    pub device_name: String,
    pub config: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportVsrTemplateRequest {
    pub path: String,
    pub include_local: bool,
    pub include_ldap: bool,
    pub include_radius: bool,
}

#[derive(Clone, Copy)]
struct TemplateOptions {
    include_local: bool,
    include_ldap: bool,
    include_radius: bool,
}

fn should_include_field(field: &FieldMeta, options: &TemplateOptions) -> bool {
    match field.category {
        FieldCategory::Core => true,
        FieldCategory::Local => options.include_local,
        FieldCategory::Ldap => options.include_ldap,
        FieldCategory::Radius => options.include_radius,
    }
}

#[tauri::command]
pub fn export_vsr_template(request: ExportVsrTemplateRequest) -> Result<(), String> {
    let options = TemplateOptions {
        include_local: request.include_local,
        include_ldap: request.include_ldap,
        include_radius: request.include_radius,
    };
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = Format::new().set_bold().set_align(FormatAlign::Center);

    let mut col_index: u16 = 0;
    for field in VSR_FIELDS.iter() {
        if DERIVED_POOL_FIELDS.contains(&field.key) {
            continue;
        }
        if !should_include_field(field, &options) {
            continue;
        }
        worksheet
            .write_with_format(0u32, col_index, field.key, &header_format)
            .map_err(|err| err.to_string())?;
        col_index += 1;
    }

    workbook.save(request.path).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn process_vsr_excel(request: ProcessExcelRequest) -> Result<ExcelAnalysis, String> {
    let ProcessExcelRequest {
        file_path,
        sheet_name,
    } = request;
    let mut workbook =
        open_workbook_auto(&file_path).map_err(|err| format!("无法打开 Excel 文件: {err}"))?;

    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Excel 文件中没有工作表".into());
    }

    let selected = sheet_name
        .filter(|name| sheet_names.contains(name))
        .unwrap_or_else(|| sheet_names[0].clone());

    let range = workbook
        .worksheet_range(&selected)
        .map_err(|err| format!("读取工作表失败: {err}"))?;

    let header_row_index =
        detect_header_row(&range).ok_or_else(|| "无法定位表头行，请检查 Excel 格式".to_string())?;

    let columns = range
        .rows()
        .nth(header_row_index)
        .map(|row| row.iter().map(data_type_to_string).collect::<Vec<_>>())
        .unwrap_or_default();

    let preview_rows = collect_preview_rows(&range, header_row_index);

    Ok(ExcelAnalysis {
        sheet_names,
        selected_sheet: selected,
        header_row_index,
        columns: columns.clone(),
        preview_rows,
        suggested_mapping: suggest_column_mapping(&columns),
        total_rows: range.height().saturating_sub(header_row_index + 1),
    })
}

#[tauri::command]
pub fn convert_vsr_entries(request: ConvertVsrRequest) -> Result<ConvertResponse, String> {
    let ConvertVsrRequest {
        file_path,
        sheet_name,
        header_row_index,
    } = request;

    let mut workbook =
        open_workbook_auto(&file_path).map_err(|err| format!("无法打开 Excel 文件: {err}"))?;
    let sheet_names = workbook.sheet_names().to_vec();
    let selected = sheet_name
        .filter(|name| sheet_names.contains(name))
        .unwrap_or_else(|| sheet_names[0].clone());

    let range = workbook
        .worksheet_range(&selected)
        .map_err(|err| format!("读取工作表失败: {err}"))?;

    let rows = extract_rows(&range, header_row_index)?;
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    for row in rows {
        match validate_entry(row) {
            Ok(entry) => entries.push(entry),
            Err(err) => errors.push(err),
        }
    }

    Ok(ConvertResponse { entries, errors })
}

#[tauri::command]
pub fn generate_vsr_configs(
    request: GenerateVsrConfigRequest,
) -> Result<Vec<VsrGeneratedConfig>, String> {
    let mut results = Vec::new();
    let options = TemplateOptions {
        include_local: request.include_local,
        include_ldap: request.include_ldap,
        include_radius: request.include_radius,
    };

    for entry in request.entries {
        let mut context = Context::new();
        context.insert("ip", &entry.ip);
        context.insert("gateway", &entry.gateway);
        context.insert(
            "vsr_username",
            &optional_value(entry.vsr_username.clone(), options.include_local),
        );
        context.insert(
            "vsr_password",
            &optional_value(entry.vsr_password.clone(), options.include_local),
        );
        context.insert(
            "monitor_username",
            &optional_value(entry.monitor_username.clone(), options.include_local),
        );
        context.insert(
            "monitor_password",
            &optional_value(entry.monitor_password.clone(), options.include_local),
        );
        context.insert(
            "ppp_username",
            &optional_value(entry.ppp_username.clone(), options.include_local),
        );
        context.insert(
            "ppp_password",
            &optional_value(entry.ppp_password.clone(), options.include_local),
        );
        context.insert("start_ip", &entry.start_ip);
        context.insert("end_ip", &entry.end_ip);
        context.insert("pool_ip_gateway", &entry.pool_ip_gateway);
        context.insert(
            "ldap_server_ip",
            &optional_value(entry.ldap_server_ip.clone(), options.include_ldap),
        );
        context.insert(
            "ldap_login_dn",
            &optional_value(entry.ldap_login_dn.clone(), options.include_ldap),
        );
        context.insert(
            "ldap_search_base_dn",
            &optional_value(entry.ldap_search_base_dn.clone(), options.include_ldap),
        );
        context.insert(
            "ldap_password",
            &optional_value(entry.ldap_password.clone(), options.include_ldap),
        );
        context.insert(
            "radius_ip",
            &optional_value(entry.radius_ip.clone(), options.include_radius),
        );
        context.insert(
            "radius_password",
            &optional_value(entry.radius_password.clone(), options.include_radius),
        );

        let rendered = tera::Tera::one_off(VSR_TEMPLATE, &context, false)
            .map_err(|err| format!("渲染模板失败: {err}"))?;

        results.push(VsrGeneratedConfig {
            device_name: entry.device_name.clone(),
            config: rendered,
        });
    }

    Ok(results)
}

#[tauri::command]
pub fn export_vsr_configs(path: String, configs: Vec<VsrGeneratedConfig>) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (col, config) in configs.iter().enumerate() {
        worksheet
            .write(0u32, col as u16, &config.device_name)
            .map_err(|err| err.to_string())?;
        for (row_offset, line) in config.config.lines().enumerate() {
            worksheet
                .write((row_offset + 1) as u32, col as u16, line)
                .map_err(|err| err.to_string())?;
        }
    }

    workbook.save(path).map_err(|err| err.to_string())
}

fn collect_preview_rows(range: &Range<Data>, header_row_index: usize) -> Vec<Vec<String>> {
    let mut preview_rows = Vec::new();
    for (idx, row) in range.rows().enumerate() {
        if idx <= header_row_index {
            continue;
        }
        if preview_rows.len() >= MAX_PREVIEW_ROWS {
            break;
        }
        let row_values = row.iter().map(data_type_to_string).collect::<Vec<_>>();
        preview_rows.push(row_values);
    }
    preview_rows
}

fn detect_header_row(range: &Range<Data>) -> Option<usize> {
    let mut best_row = None;
    let mut best_score = 0usize;

    for (idx, row) in range.rows().enumerate().take(20) {
        let columns = row.iter().map(data_type_to_string).collect::<Vec<_>>();
        let mapping = suggest_column_mapping(&columns);
        if mapping.len() > best_score {
            best_score = mapping.len();
            best_row = Some(idx);
        }
        if mapping.len() == VSR_FIELDS.len() {
            return Some(idx);
        }
    }

    best_row
}

fn suggest_column_mapping(columns: &[String]) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (field, patterns) in COLUMN_PATTERNS {
        let mut best_score = 0.0;
        let mut best_column: Option<String> = None;
        for column in columns {
            let normalized = normalize(column);
            if normalized == *field {
                best_score = 1.0;
                best_column = Some(column.clone());
                break;
            }
            for pattern in *patterns {
                let score = if normalized == pattern.to_lowercase() {
                    1.0
                } else {
                    normalized_levenshtein(&normalized, &pattern.to_lowercase())
                };
                if score > best_score {
                    best_score = score;
                    best_column = Some(column.clone());
                }
            }
        }
        if best_score > 0.75 {
            if let Some(column) = best_column {
                result.insert((*field).to_string(), column);
            }
        }
    }
    result
}

fn extract_rows(
    range: &Range<Data>,
    header_row_index: usize,
) -> Result<Vec<VsrDeviceEntry>, String> {
    let header_row = range
        .rows()
        .nth(header_row_index)
        .ok_or_else(|| format!("无法找到第 {} 行表头", header_row_index + 1))?;

    let columns = header_row
        .iter()
        .map(data_type_to_string)
        .collect::<Vec<_>>();
    if columns.is_empty() {
        return Err("检测到空表头行，无法解析列映射".to_string());
    }

    let column_mapping = suggest_column_mapping(&columns);

    let mut column_indices = HashMap::new();
    for (idx, name) in columns.iter().enumerate() {
        if name.is_empty() {
            continue;
        }
        column_indices.insert(normalize(name), idx);
    }

    for field in VSR_FIELDS.iter().filter(|f| f.required) {
        if !column_mapping.contains_key(field.key) {
            return Err(format!("无法根据表头匹配到列: {}", field.key));
        }
    }

    let mut rows = Vec::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx <= header_row_index {
            continue;
        }
        if row.iter().all(|cell| data_type_to_string(cell).is_empty()) {
            continue;
        }

        let mut values = HashMap::new();
        for (field_key, column_name) in column_mapping.iter() {
            if let Some(col_index) = column_indices.get(&normalize(column_name)) {
                if let Some(cell) = row.get(*col_index) {
                    values.insert(field_key.clone(), data_type_to_string(cell));
                }
            }
        }

        rows.push(VsrDeviceEntry {
            device_name: values.get("device_name").cloned().unwrap_or_default(),
            ip: values.get("ip").cloned().unwrap_or_default(),
            gateway: values.get("gateway").cloned().unwrap_or_default(),
            vsr_username: normalize_optional(values.get("vsr_username")),
            vsr_password: normalize_optional(values.get("vsr_password")),
            monitor_username: normalize_optional(values.get("monitor_username")),
            monitor_password: normalize_optional(values.get("monitor_password")),
            ppp_username: normalize_optional(values.get("ppp_username")),
            ppp_password: normalize_optional(values.get("ppp_password")),
            pool_cidr: normalize_optional(values.get("pool_cidr")),
            start_ip: values.get("start_ip").cloned().unwrap_or_default(),
            end_ip: values.get("end_ip").cloned().unwrap_or_default(),
            pool_ip_gateway: values.get("pool_ip_gateway").cloned().unwrap_or_default(),
            ldap_server_ip: normalize_optional(values.get("ldap_server_ip")),
            ldap_login_dn: normalize_optional(values.get("ldap_login_dn")),
            ldap_search_base_dn: normalize_optional(values.get("ldap_search_base_dn")),
            ldap_password: normalize_optional(values.get("ldap_password")),
            radius_ip: normalize_optional(values.get("radius_ip")),
            radius_password: normalize_optional(values.get("radius_password")),
            row_index: row_idx + 1,
        });
    }

    for row in rows.iter_mut() {
        if let Some(cidr) = row.pool_cidr.clone() {
            if !row.start_ip.trim().is_empty()
                && !row.end_ip.trim().is_empty()
                && !row.pool_ip_gateway.trim().is_empty()
            {
                continue;
            }

            if let Some(calc) = compute_pool_from_cidr(&cidr) {
                if row.pool_ip_gateway.trim().is_empty() {
                    row.pool_ip_gateway = calc.gateway;
                }
                if row.start_ip.trim().is_empty() {
                    row.start_ip = calc.start;
                }
                if row.end_ip.trim().is_empty() {
                    row.end_ip = calc.end;
                }
            }
        }
    }

    Ok(rows)
}

fn compute_pool_from_cidr(cidr: &str) -> Option<PoolComputation> {
    let parsed: Ipv4Net = cidr.trim().parse().ok()?;
    let network = u32::from(parsed.network());
    let broadcast = u32::from(parsed.broadcast());

    // 至少需要网关 + 起始 + 结束三个地址
    if broadcast.saturating_sub(network) < 3 {
        return None;
    }

    let gateway = network + 1;
    let start = gateway + 1;
    let end = broadcast.saturating_sub(1);

    if start > end {
        return None;
    }

    Some(PoolComputation {
        gateway: Ipv4Addr::from(gateway).to_string(),
        start: Ipv4Addr::from(start).to_string(),
        end: Ipv4Addr::from(end).to_string(),
    })
}

fn validate_entry(entry: VsrDeviceEntry) -> Result<VsrDeviceEntry, String> {
    let mut missing = Vec::new();
    if entry.device_name.is_empty() {
        missing.push("device_name");
    }
    if entry.ip.is_empty() {
        missing.push("ip");
    }
    if entry.gateway.is_empty() {
        missing.push("gateway");
    }
    if entry.start_ip.is_empty() {
        missing.push("start_ip");
    }
    if entry.end_ip.is_empty() {
        missing.push("end_ip");
    }
    if entry.pool_ip_gateway.is_empty() {
        missing.push("pool_ip_gateway");
    }

    if missing.is_empty() {
        Ok(entry)
    } else {
        Err(format!(
            "第 {} 行缺少必填字段: {}",
            entry.row_index,
            missing.join(", ")
        ))
    }
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
        Data::Empty => String::new(),
        Data::Error(_) => String::new(),
        other => other.to_string(),
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalize_optional(value: Option<&String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn optional_value(value: Option<String>, enabled: bool) -> String {
    if !enabled {
        String::new()
    } else {
        value.unwrap_or_default()
    }
}
