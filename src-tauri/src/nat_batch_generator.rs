use crate::isp_manager::{
    find_isp_for_ip, load_elastic_ip_mapping_internal, load_isp_data_internal,
};
use calamine::{open_workbook_auto, Data, Range, Reader};
use regex::Regex;
use rust_xlsxwriter::{Format, FormatAlign, Workbook};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::sync::OnceLock;
use strsim::normalized_levenshtein;

const REQUIRED_FIELDS: [&str; 5] = ["协议", "主机IP", "内网端口", "外网IP", "外网端口"];
const MAX_PREVIEW_ROWS: usize = 100;
const DEFAULT_PORT_SPLIT_SPAN: u16 = 1000;
const TEMPLATE_HEADERS: [&str; 5] = ["协议", "主机IP", "内网端口", "外网IP", "外网端口"];
const TEMPLATE_SAMPLE_ROWS: [[&str; 5]; 3] = [
    ["TCP", "192.168.1.100", "80", "222.240.138.4", "80"],
    ["UDP", "192.168.1.101", "53", "222.240.138.5", "53"],
    ["ANY", "192.168.1.102", "", "222.240.138.6", ""],
];

static COLUMN_PATTERNS: &[(&str, &[&str])] = &[
    ("协议", &["protocol", "type", "协议", "类型", "协议类型"]),
    (
        "主机IP",
        &[
            "host_ip",
            "internal_ip",
            "inside_ip",
            "server_ip",
            "主机ip",
            "内网ip",
            "服务器ip",
            "主机地址",
            "内网地址",
            "服务器地址",
            "主机ip地址",
            "内网ip地址",
            "服务器ip地址",
        ],
    ),
    (
        "内网端口",
        &[
            "host_port",
            "internal_port",
            "inside_port",
            "server_port",
            "主机端口",
            "内网端口",
            "服务器端口",
            "端口号",
            "内网端口号",
            "服务器端口号",
            "主机端口号",
            "内网映射端口号",
            "主机映射端口号",
            "服务器映射端口号",
        ],
    ),
    (
        "外网IP",
        &[
            "public_ip",
            "external_ip",
            "outside_ip",
            "公网ip",
            "外网ip",
            "外部ip",
            "互联网ip",
            "外网地址",
            "外网ip地址",
            "公网地址",
            "公网ip地址",
            "互联网地址",
            "互联网ip地址",
        ],
    ),
    (
        "外网端口",
        &[
            "public_port",
            "external_port",
            "outside_port",
            "公网端口",
            "外网端口",
            "外部端口",
            "互联网端口",
            "外网映射端口号",
            "公网映射端口号",
            "互联网映射端口号",
        ],
    ),
];

fn ip_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\b(\d{1,3}(?:\.\d{1,3}){3})\b").expect("编译正则失败"))
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

fn normalize_column_name(value: &str) -> String {
    value.trim().to_lowercase()
}

fn suggest_column_mapping(columns: &[String]) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (field, patterns) in COLUMN_PATTERNS {
        let mut best_score = 0.0_f64;
        let mut best_column: Option<String> = None;
        for column in columns {
            let column_normalized = normalize_column_name(column);
            for pattern in (*patterns).iter() {
                let pattern_normalized = pattern.trim().to_lowercase();
                let score = if column_normalized == pattern_normalized {
                    1.0
                } else {
                    normalized_levenshtein(&column_normalized, &pattern_normalized)
                };
                if score > best_score {
                    best_score = score;
                    best_column = Some(column.clone());
                }
            }
        }
        if best_score > 0.7 {
            if let Some(column) = best_column {
                result.insert((*field).to_string(), column);
            }
        }
    }
    result
}

fn detect_header_row(range: &Range<Data>) -> Option<usize> {
    let mut best_match_row = None;
    let mut best_match_count = 0usize;

    for (idx, row) in range.rows().enumerate().take(20) {
        let columns = row.iter().map(data_type_to_string).collect::<Vec<_>>();
        let mapping = suggest_column_mapping(&columns);
        let matched = mapping.len();
        if matched > best_match_count {
            best_match_count = matched;
            best_match_row = Some(idx);
        }
        if matched == REQUIRED_FIELDS.len() {
            return Some(idx);
        }
    }

    if best_match_count == 0 {
        None
    } else {
        best_match_row
    }
}

fn default_sheet_name(sheet_names: &[String]) -> Option<String> {
    if sheet_names.is_empty() {
        return None;
    }

    for candidate in ["映射", "需求"] {
        if let Some(name) = sheet_names.iter().find(|sheet| sheet.contains(candidate)) {
            return Some(name.clone());
        }
    }

    sheet_names.first().cloned()
}

fn process_public_ips(value: &str) -> Vec<String> {
    if value.trim().is_empty() {
        return Vec::new();
    }
    ip_regex()
        .find_iter(&value.replace('\n', " "))
        .map(|m| m.as_str().trim().to_string())
        .collect()
}

fn parse_ipv4(value: &str) -> Result<Ipv4Addr, String> {
    value
        .trim()
        .parse::<Ipv4Addr>()
        .map_err(|err| format!("无效的 IPv4 地址: {value}, 错误: {err}"))
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

impl PortRange {
    pub fn len(&self) -> u32 {
        (self.end as u32).saturating_sub(self.start as u32) + 1
    }

    pub fn is_range(&self) -> bool {
        self.start != self.end
    }
}

fn parse_port_range(value: &str) -> Result<PortRange, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("端口号不能为空".to_string());
    }

    let parts = if trimmed.contains('-') {
        trimmed.split('-').collect::<Vec<_>>()
    } else if trimmed.contains(':') {
        trimmed.split(':').collect::<Vec<_>>()
    } else {
        vec![trimmed]
    };

    if parts.is_empty() {
        return Err("端口格式错误".to_string());
    }

    let parse_single = |p: &str| -> Result<u16, String> {
        let num = p
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("无效的端口号: {p}"))?;
        if !(1..=65535).contains(&num) {
            return Err(format!("端口号必须在 1-65535 之间: {num}"));
        }
        Ok(num as u16)
    };

    if parts.len() == 1 {
        let port = parse_single(parts[0])?;
        return Ok(PortRange {
            start: port,
            end: port,
        });
    }

    if parts.len() != 2 {
        return Err("端口范围格式错误，应为 起始端口-结束端口".to_string());
    }

    let start = parse_single(parts[0])?;
    let end = parse_single(parts[1])?;

    if start > end {
        return Err(format!("起始端口 {start} 不能大于结束端口 {end}"));
    }

    Ok(PortRange { start, end })
}

fn split_port_range_port(range: PortRange, max_span: u16) -> Vec<PortRange> {
    if max_span == 0 {
        return vec![range];
    }

    let mut result = Vec::new();
    let mut current_start = range.start;
    while current_start <= range.end {
        let current_end = (current_start as u32 + max_span as u32 - 1).min(range.end as u32) as u16;
        result.push(PortRange {
            start: current_start,
            end: current_end,
        });
        if current_end == u16::MAX {
            break;
        }
        current_start = current_end.saturating_add(1);
    }
    result
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

#[tauri::command]
pub fn process_excel_data(request: ProcessExcelRequest) -> Result<ExcelAnalysis, String> {
    let ProcessExcelRequest {
        file_path,
        sheet_name,
    } = request;
    let mut workbook =
        open_workbook_auto(&file_path).map_err(|err| format!("无法打开 Excel 文件: {err}"))?;

    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Excel 文件中没有工作表".to_string());
    }

    let selected = if let Some(name) = sheet_name {
        name
    } else {
        default_sheet_name(&sheet_names).ok_or_else(|| "无法确定工作表".to_string())?
    };

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

    let suggested_mapping = suggest_column_mapping(&columns);

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

    let total_rows = range.height().saturating_sub(header_row_index + 1);

    Ok(ExcelAnalysis {
        sheet_names,
        selected_sheet: selected,
        header_row_index,
        columns,
        preview_rows,
        suggested_mapping,
        total_rows,
    })
}

#[derive(Default)]
struct RawNatRow {
    protocol: String,
    internal_ip: String,
    internal_port: Option<String>,
    public_ip: String,
    public_port: Option<String>,
    row_index: usize,
}

fn extract_rows_from_excel(
    range: &Range<Data>,
    header_row_index: usize,
    column_mapping: &HashMap<String, String>,
) -> Result<Vec<RawNatRow>, String> {
    let header_row = range
        .rows()
        .nth(header_row_index)
        .ok_or_else(|| format!("无法找到第 {} 行表头", header_row_index + 1))?;

    let mut column_indices = HashMap::new();
    for (idx, cell) in header_row.iter().enumerate() {
        let name = data_type_to_string(cell);
        if name.is_empty() {
            continue;
        }
        column_indices.insert(normalize_column_name(&name), idx);
    }

    for &required in REQUIRED_FIELDS.iter() {
        if !column_mapping.contains_key(required) {
            return Err(format!("缺少列映射: {required}"));
        }
    }

    let mapped_indices = column_mapping
        .iter()
        .map(|(field, column)| {
            let normalized = normalize_column_name(column);
            let index = column_indices
                .get(&normalized)
                .copied()
                .ok_or_else(|| format!("无法在表头中找到列 \"{column}\" (字段: {field})"))?;
            Ok(((*field).to_string(), index))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;

    let mut rows = Vec::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx <= header_row_index {
            continue;
        }
        let fetch_value = |field: &str| -> String {
            mapped_indices
                .get(field)
                .and_then(|index| row.get(*index))
                .map(data_type_to_string)
                .unwrap_or_default()
        };

        let protocol = fetch_value("协议");
        let internal_ip = fetch_value("主机IP");
        let internal_port = fetch_value("内网端口");
        let public_ip = fetch_value("外网IP");
        let public_port = fetch_value("外网端口");

        // 若关键字段均为空则跳过
        if protocol.trim().is_empty()
            && internal_ip.trim().is_empty()
            && public_ip.trim().is_empty()
        {
            continue;
        }

        rows.push(RawNatRow {
            protocol,
            internal_ip,
            internal_port: if internal_port.trim().is_empty() {
                None
            } else {
                Some(internal_port)
            },
            public_ip,
            public_port: if public_port.trim().is_empty() {
                None
            } else {
                Some(public_port)
            },
            row_index: row_idx + 1,
        });
    }

    Ok(rows)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualEntry {
    pub protocol: String,
    pub internal_ip: String,
    pub internal_port: Option<String>,
    pub public_ip: String,
    pub public_port: Option<String>,
}

fn convert_manual_entries(rows: Vec<ManualEntry>) -> Vec<RawNatRow> {
    rows.into_iter()
        .enumerate()
        .map(|(idx, entry)| RawNatRow {
            protocol: entry.protocol,
            internal_ip: entry.internal_ip,
            internal_port: entry.internal_port.filter(|value| !value.trim().is_empty()),
            public_ip: entry.public_ip,
            public_port: entry.public_port.filter(|value| !value.trim().is_empty()),
            row_index: idx + 1,
        })
        .collect()
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NatEntry {
    pub row_index: usize,
    pub protocol: String,
    pub internal_ip: String,
    pub internal_port_start: Option<u16>,
    pub internal_port_end: Option<u16>,
    pub public_ips: Vec<String>,
    pub public_port_start: Option<u16>,
    pub public_port_end: Option<u16>,
    pub is_port_range: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertResponse {
    pub entries: Vec<NatEntry>,
    pub errors: Vec<String>,
}

fn validate_rows(rows: Vec<RawNatRow>) -> ConvertResponse {
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    for row in rows {
        let mut row_errors = Vec::new();
        let row_label = format!("第 {} 行", row.row_index);

        let protocol_upper = row.protocol.trim().to_uppercase();
        if protocol_upper.is_empty() {
            row_errors.push("协议不能为空".to_string());
        } else if !matches!(protocol_upper.as_str(), "TCP" | "UDP" | "ANY") {
            row_errors.push(format!("无效的协议类型: {}", row.protocol.trim()));
        }

        let internal_ip = match parse_ipv4(&row.internal_ip) {
            Ok(ip) => ip,
            Err(err) => {
                row_errors.push(err);
                // 无法继续验证该行
                errors.push(format!("{row_label}: {}", row_errors.join("；")));
                continue;
            }
        };

        let mut public_ips = Vec::new();
        let ip_candidates = process_public_ips(&row.public_ip);
        if ip_candidates.is_empty() {
            row_errors.push("未找到有效的公网 IP".to_string());
        } else {
            for ip in ip_candidates {
                match parse_ipv4(&ip) {
                    Ok(parsed) => public_ips.push(parsed.to_string()),
                    Err(err) => row_errors.push(err),
                }
            }
        }

        let (internal_range, public_range, is_port_range) = if protocol_upper == "ANY" {
            (None, None, false)
        } else {
            let internal_port_value = match &row.internal_port {
                Some(value) if !value.trim().is_empty() => value.clone(),
                _ => {
                    row_errors.push("TCP/UDP 协议必须指定内网端口".to_string());
                    String::new()
                }
            };
            let public_port_value = match &row.public_port {
                Some(value) if !value.trim().is_empty() => value.clone(),
                _ => {
                    row_errors.push("TCP/UDP 协议必须指定外网端口".to_string());
                    String::new()
                }
            };

            if internal_port_value.is_empty() || public_port_value.is_empty() {
                (None, None, false)
            } else {
                match (
                    parse_port_range(&internal_port_value),
                    parse_port_range(&public_port_value),
                ) {
                    (Ok(internal), Ok(public)) => {
                        if internal.len() != public.len() {
                            row_errors.push(format!(
                                "内网端口范围({}-{})与外网端口范围({}-{})数量不匹配",
                                internal.start, internal.end, public.start, public.end
                            ));
                            (None, None, false)
                        } else {
                            (Some(internal), Some(public), internal.is_range())
                        }
                    }
                    (Err(err), _) | (_, Err(err)) => {
                        row_errors.push(err);
                        (None, None, false)
                    }
                }
            }
        };

        if !row_errors.is_empty() {
            errors.push(format!("{row_label}: {}", row_errors.join("；")));
            continue;
        }

        entries.push(NatEntry {
            row_index: row.row_index,
            protocol: protocol_upper.clone(),
            internal_ip: internal_ip.to_string(),
            internal_port_start: internal_range.map(|range| range.start),
            internal_port_end: internal_range.map(|range| range.end),
            public_ips,
            public_port_start: public_range.map(|range| range.start),
            public_port_end: public_range.map(|range| range.end),
            is_port_range,
        });
    }

    ConvertResponse { entries, errors }
}

#[derive(Deserialize)]
#[serde(tag = "source", rename_all = "camelCase")]
pub enum ConvertInputRequest {
    Excel {
        file_path: String,
        sheet_name: Option<String>,
        header_row_index: usize,
        column_mapping: HashMap<String, String>,
    },
    Manual {
        rows: Vec<ManualEntry>,
    },
}

#[tauri::command]
pub fn convert_excel_to_entries(request: ConvertInputRequest) -> Result<ConvertResponse, String> {
    match request {
        ConvertInputRequest::Excel {
            file_path,
            sheet_name,
            header_row_index,
            column_mapping,
        } => {
            let mut workbook = open_workbook_auto(&file_path)
                .map_err(|err| format!("无法打开 Excel 文件: {err}"))?;

            let selected = if let Some(name) = sheet_name {
                name
            } else {
                let sheet_names = workbook.sheet_names().to_vec();
                default_sheet_name(&sheet_names).ok_or_else(|| "无法确定工作表".to_string())?
            };

            let range = workbook
                .worksheet_range(&selected)
                .map_err(|err| format!("读取工作表失败: {err}"))?;

            let rows = extract_rows_from_excel(&range, header_row_index, &column_mapping)?;
            let result = validate_rows(rows);

            if result.entries.is_empty() {
                if result.errors.is_empty() {
                    Err("没有找到有效的数据".to_string())
                } else {
                    Err(result.errors.join("；"))
                }
            } else {
                Ok(result)
            }
        }
        ConvertInputRequest::Manual { rows } => {
            let raw_rows = convert_manual_entries(rows);
            let result = validate_rows(raw_rows);
            if result.entries.is_empty() {
                if result.errors.is_empty() {
                    Err("没有找到有效的数据".to_string())
                } else {
                    Err(result.errors.join("；"))
                }
            } else {
                Ok(result)
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Huawei,
    H3c,
}

impl Default for DeviceType {
    fn default() -> Self {
        DeviceType::Huawei
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateNatCommandsRequest {
    pub entries: Vec<NatEntry>,
    #[serde(default)]
    pub use_elastic_ip: bool,
    #[serde(default)]
    pub device_type: DeviceType,
    pub vrrp_id: Option<u16>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateNatCommandsResponse {
    pub commands: Vec<String>,
    pub missing_elastic_ips: Vec<String>,
}

fn build_huawei_command(
    entry: &NatEntry,
    public_ip: &str,
    isp_prefix: &str,
    elastic_ip: Option<&str>,
) -> Option<String> {
    let inside_ip = elastic_ip.unwrap_or(&entry.internal_ip);
    if entry.protocol == "ANY" {
        let name = format!("{isp_prefix}{}{}", entry.protocol, entry.internal_ip);
        Some(format!(
            "nat server {name} global {public_ip} inside {inside_ip} no-reverse"
        ))
    } else {
        let protocol_lower = entry.protocol.to_lowercase();
        let (internal_start, internal_end) = (
            entry.internal_port_start?,
            entry
                .internal_port_end
                .unwrap_or(entry.internal_port_start.unwrap()),
        );
        let (public_start, public_end) = (
            entry.public_port_start?,
            entry
                .public_port_end
                .unwrap_or(entry.public_port_start.unwrap()),
        );

        if internal_start == internal_end {
            let name = format!(
                "{isp_prefix}{}{}:{}",
                entry.protocol, entry.internal_ip, internal_start
            );
            Some(format!(
                "nat server {name} protocol {protocol_lower} global {public_ip} {public_start} inside {inside_ip} {internal_start} no-reverse"
            ))
        } else {
            let name = format!(
                "{isp_prefix}{}{}:{}-{}",
                entry.protocol, entry.internal_ip, internal_start, internal_end
            );
            Some(format!(
                "nat server {name} protocol {protocol_lower} global {public_ip} {public_start} {public_end} inside {inside_ip} {internal_start} {internal_end} no-reverse"
            ))
        }
    }
}

fn build_h3c_command(
    entry: &NatEntry,
    public_ip: &str,
    isp_prefix: &str,
    elastic_ip: Option<&str>,
    vrrp_id: u16,
) -> Option<String> {
    let inside_ip = elastic_ip.unwrap_or(&entry.internal_ip);
    if entry.protocol == "ANY" {
        let description = format!("{isp_prefix}{}{}", entry.protocol, entry.internal_ip);
        Some(format!(
            "nat server global {public_ip} inside {inside_ip} vrrp {vrrp_id} description {description}"
        ))
    } else {
        let protocol_lower = entry.protocol.to_lowercase();
        let (internal_start, internal_end) = (
            entry.internal_port_start?,
            entry
                .internal_port_end
                .unwrap_or(entry.internal_port_start.unwrap()),
        );
        let (public_start, public_end) = (
            entry.public_port_start?,
            entry
                .public_port_end
                .unwrap_or(entry.public_port_start.unwrap()),
        );

        if internal_start == internal_end {
            let description = format!(
                "{isp_prefix}{}{}:{}",
                entry.protocol, entry.internal_ip, internal_start
            );
            Some(format!(
                "nat server protocol {protocol_lower} global {public_ip} {public_start} inside {inside_ip} {internal_start} vrrp {vrrp_id} description {description}"
            ))
        } else {
            let description = format!(
                "{isp_prefix}{}{}:{}-{}",
                entry.protocol, entry.internal_ip, internal_start, internal_end
            );
            Some(format!(
                "nat server protocol {protocol_lower} global {public_ip} {public_start} {public_end} inside {inside_ip} {internal_start} {internal_end} vrrp {vrrp_id} description {description}"
            ))
        }
    }
}

#[tauri::command]
pub fn generate_nat_commands(
    request: GenerateNatCommandsRequest,
) -> Result<GenerateNatCommandsResponse, String> {
    let elastic_mapping = load_elastic_ip_mapping_internal()?;
    let isp_data = load_isp_data_internal()?;

    let mut commands = Vec::new();
    let mut missing_elastic = HashSet::new();

    if matches!(request.device_type, DeviceType::H3c) && request.vrrp_id.is_none() {
        return Err("H3C 设备必须指定 VRRP ID".to_string());
    }
    let vrrp_id = request.vrrp_id.unwrap_or_default();

    for entry in request.entries {
        let elastic_ip = if request.use_elastic_ip {
            match elastic_mapping.get(&entry.internal_ip) {
                Some(value) => Some(value.as_str()),
                None => {
                    missing_elastic.insert(entry.internal_ip.clone());
                    None
                }
            }
        } else {
            None
        };

        for public_ip in &entry.public_ips {
            let isp_prefix = match parse_ipv4(public_ip) {
                Ok(ip) => find_isp_for_ip(&ip, &isp_data)
                    .map(|(isp, _)| format!("{}_", isp))
                    .unwrap_or_default(),
                Err(_) => String::new(),
            };

            let command = match request.device_type {
                DeviceType::Huawei => {
                    build_huawei_command(&entry, public_ip, &isp_prefix, elastic_ip)
                }
                DeviceType::H3c => {
                    build_h3c_command(&entry, public_ip, &isp_prefix, elastic_ip, vrrp_id)
                }
            };

            if let Some(cmd) = command {
                commands.push(cmd);
            }
        }
    }

    let mut missing_list = missing_elastic.into_iter().collect::<Vec<_>>();
    missing_list.sort();

    Ok(GenerateNatCommandsResponse {
        commands,
        missing_elastic_ips: missing_list,
    })
}

#[tauri::command]
pub fn export_nat_template(path: String) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let header_format = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);

    worksheet
        .set_row_height(0, 22.0)
        .map_err(|err| format!("设置表头行高失败: {err}"))?;

    for (col, header) in TEMPLATE_HEADERS.iter().enumerate() {
        worksheet
            .set_column_width(col as u16, 18.0)
            .map_err(|err| format!("设置列宽失败: {err}"))?;
        worksheet
            .write_string_with_format(0, col as u16, *header, &header_format)
            .map_err(|err| format!("写入表头失败: {err}"))?;
    }

    for (row_idx, row) in TEMPLATE_SAMPLE_ROWS.iter().enumerate() {
        for (col_idx, value) in row.iter().enumerate() {
            worksheet
                .write_string((row_idx + 1) as u32, col_idx as u16, *value)
                .map_err(|err| format!("写入示例数据失败: {err}"))?;
        }
    }

    workbook
        .save(path)
        .map_err(|err| format!("保存模板失败: {err}"))?;
    Ok(())
}

#[tauri::command]
pub fn export_nat_commands(path: String, commands: Vec<String>) -> Result<(), String> {
    let content = commands.join("\n");
    std::fs::write(&path, content).map_err(|err| format!("写入文件失败: {err}"))?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitPortRangesRequest {
    pub start_port: u16,
    pub end_port: u16,
    pub max_span: Option<u16>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitPortRangesResponse {
    pub ranges: Vec<PortRange>,
}

#[tauri::command]
pub fn split_port_ranges(
    request: SplitPortRangesRequest,
) -> Result<SplitPortRangesResponse, String> {
    if request.start_port == 0 || request.end_port == 0 {
        return Err("端口号必须大于 0".to_string());
    }
    if request.start_port > request.end_port {
        return Err("起始端口不能大于结束端口".to_string());
    }

    let range = PortRange {
        start: request.start_port,
        end: request.end_port,
    };
    let max_span = request.max_span.unwrap_or(DEFAULT_PORT_SPLIT_SPAN).max(1);
    let ranges = split_port_range_port(range, max_span);

    Ok(SplitPortRangesResponse { ranges })
}
