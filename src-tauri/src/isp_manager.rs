use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use serde_yaml::{self, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use zip::ZipArchive;

const CONFIG_DIR_NAME: &str = "nat配置工具";
const ELASTIC_IP_FILENAME: &str = "elastic_ip_config.yaml";
const ISP_DATA_FILENAME: &str = "isp_data.yaml";
const DEFAULT_SCAN_SIZE: u32 = 256;
const GITHUB_ZIP_URL: &str =
    "https://github.com/gaoyifan/china-operator-ip/archive/refs/heads/ip-lists.zip";

#[derive(Debug)]
struct ConfigPaths {
    elastic_ip_file: PathBuf,
    isp_data_file: PathBuf,
}

fn resolve_paths() -> Result<ConfigPaths, String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| "无法定位用户主目录，无法初始化配置目录".to_string())?;

    let config_dir = home_dir.join(".config").join(CONFIG_DIR_NAME);
    fs::create_dir_all(&config_dir)
        .map_err(|err| format!("创建配置目录失败: {config_dir:?}, 错误: {err}"))?;

    let elastic_ip_file = config_dir.join(ELASTIC_IP_FILENAME);
    let isp_data_file = config_dir.join(ISP_DATA_FILENAME);

    Ok(ConfigPaths {
        elastic_ip_file,
        isp_data_file,
    })
}

fn read_text_file(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("读取文件失败: {path:?}, 错误: {err}")),
    }
}

fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| format!("无法确定文件所在目录: {path:?}"))?;
    fs::create_dir_all(dir).map_err(|err| format!("创建目录失败: {dir:?}, 错误: {err}"))?;

    let mut temp_file =
        NamedTempFile::new_in(dir).map_err(|err| format!("创建临时文件失败: {err}"))?;
    use std::io::Write;
    temp_file
        .write_all(content.as_bytes())
        .map_err(|err| format!("写入临时文件失败: {err}"))?;
    temp_file
        .persist(path)
        .map_err(|err| format!("写入配置文件失败: {err}"))?;
    Ok(())
}

pub(crate) fn load_elastic_ip_mapping_internal() -> Result<HashMap<String, String>, String> {
    let paths = resolve_paths()?;
    let maybe_content = read_text_file(&paths.elastic_ip_file)?;
    let Some(content) = maybe_content else {
        return Ok(HashMap::new());
    };

    if content.trim().is_empty() {
        return Ok(HashMap::new());
    }

    match serde_yaml::from_str::<Value>(&content) {
        Ok(Value::Mapping(mapping)) => {
            let mut result = HashMap::new();
            for (key, value) in mapping {
                if let (Value::String(k), Value::String(v)) = (key, value) {
                    result.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
            Ok(result)
        }
        Ok(Value::Null) => Ok(HashMap::new()),
        Ok(other) => Err(format!("弹性 IP 配置格式不正确: {other:?}")),
        Err(err) => Err(format!("解析弹性 IP 配置失败: {err}")),
    }
}

pub(crate) fn save_elastic_ip_mapping_internal(
    mapping: &HashMap<String, String>,
) -> Result<(), String> {
    let paths = resolve_paths()?;
    let mut ordered: BTreeMap<&String, &String> = BTreeMap::new();
    for (key, value) in mapping {
        ordered.insert(key, value);
    }

    let serialized =
        serde_yaml::to_string(&ordered).map_err(|err| format!("序列化弹性 IP 配置失败: {err}"))?;
    write_atomic(&paths.elastic_ip_file, &serialized)
}

pub(crate) fn load_isp_data_internal() -> Result<HashMap<String, Vec<String>>, String> {
    let paths = resolve_paths()?;
    let maybe_content = read_text_file(&paths.isp_data_file)?;
    let Some(content) = maybe_content else {
        return Ok(HashMap::new());
    };

    if content.trim().is_empty() {
        return Ok(HashMap::new());
    }

    match serde_yaml::from_str::<Value>(&content) {
        Ok(Value::Mapping(mapping)) => {
            let mut result = HashMap::new();
            for (key, value) in mapping {
                if let Value::String(k) = key {
                    let ranges = match value {
                        Value::Sequence(seq) => seq
                            .into_iter()
                            .filter_map(|item| match item {
                                Value::String(v) => Some(v.trim().to_string()),
                                _ => None,
                            })
                            .collect::<Vec<_>>(),
                        Value::String(v) => vec![v.trim().to_string()],
                        _ => Vec::new(),
                    };
                    result.insert(k.trim().to_string(), ranges);
                }
            }
            Ok(result)
        }
        Ok(Value::Null) => Ok(HashMap::new()),
        Ok(other) => Err(format!("运营商数据格式不正确: {other:?}")),
        Err(err) => Err(format!("解析运营商数据失败: {err}")),
    }
}

pub(crate) fn save_isp_data_internal(data: &HashMap<String, Vec<String>>) -> Result<(), String> {
    let paths = resolve_paths()?;
    let mut ordered = BTreeMap::new();
    for (key, values) in data {
        ordered.insert(key, values);
    }
    let serialized =
        serde_yaml::to_string(&ordered).map_err(|err| format!("序列化运营商数据失败: {err}"))?;
    write_atomic(&paths.isp_data_file, &serialized)
}

fn parse_ipv4(value: &str) -> Result<Ipv4Addr, String> {
    value
        .trim()
        .parse::<Ipv4Addr>()
        .map_err(|err| format!("无效的 IPv4 地址: {value}, 错误: {err}"))
}

fn build_elastic_entry_map() -> Result<(HashMap<String, String>, ConfigPaths), String> {
    let paths = resolve_paths()?;
    let mapping = load_elastic_ip_mapping_internal()?;
    Ok((mapping, paths))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElasticIpMappingRequest {
    pub internal_ip: String,
    pub elastic_ip: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ElasticMappingEntry {
    pub internal_ip: String,
    pub elastic_ip: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkElasticIpResult {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkElasticIpRequest {
    pub entries: Vec<ElasticIpMappingRequest>,
    #[serde(default)]
    pub overwrite_existing: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextAvailableElasticIpRequest {
    pub start_ip: String,
    pub scan_size: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextAvailableElasticIpResponse {
    pub next_ip: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IspUpdateResult {
    pub dx_count: usize,
    pub lt_count: usize,
    pub yd_count: usize,
    pub other_count: usize,
    pub total: usize,
    pub saved_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIspDataRequest {
    pub data: HashMap<String, Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectIspResponse {
    pub isp: Option<String>,
    pub matched_range: Option<String>,
}

fn summarize_isp_data(data: &HashMap<String, Vec<String>>, saved_path: String) -> IspUpdateResult {
    let mut dx_count = 0usize;
    let mut lt_count = 0usize;
    let mut yd_count = 0usize;
    let mut other_count = 0usize;

    for (key, ranges) in data {
        let count = ranges.len();
        match key.as_str() {
            "DX" => dx_count += count,
            "LT" => lt_count += count,
            "YD" => yd_count += count,
            _ => other_count += count,
        }
    }

    let total = dx_count + lt_count + yd_count + other_count;

    IspUpdateResult {
        dx_count,
        lt_count,
        yd_count,
        other_count,
        total,
        saved_path,
    }
}

pub(crate) fn find_isp_for_ip(
    ip: &Ipv4Addr,
    data: &HashMap<String, Vec<String>>,
) -> Option<(String, String)> {
    for (isp, ranges) in data {
        for range in ranges {
            if let Ok(net) = range.parse::<IpNet>() {
                if net.contains(&IpAddr::V4(*ip)) {
                    return Some((isp.clone(), range.clone()));
                }
            }
        }
    }
    None
}

#[tauri::command]
pub fn add_elastic_ip_mapping(
    request: ElasticIpMappingRequest,
) -> Result<ElasticMappingEntry, String> {
    let (mut mapping, _) = build_elastic_entry_map()?;

    let internal = parse_ipv4(&request.internal_ip)?.to_string();
    let elastic = parse_ipv4(&request.elastic_ip)?.to_string();

    mapping.insert(internal.clone(), elastic.clone());
    save_elastic_ip_mapping_internal(&mapping)?;

    Ok(ElasticMappingEntry {
        internal_ip: internal,
        elastic_ip: elastic,
    })
}

#[tauri::command]
pub fn bulk_add_elastic_ip_mappings(
    request: BulkElasticIpRequest,
) -> Result<BulkElasticIpResult, String> {
    let (mut mapping, _) = build_elastic_entry_map()?;
    let mut added = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;

    for entry in request.entries {
        let internal_ip = match parse_ipv4(&entry.internal_ip) {
            Ok(ip) => ip.to_string(),
            Err(_) => {
                skipped += 1;
                continue;
            }
        };
        let elastic_ip = match parse_ipv4(&entry.elastic_ip) {
            Ok(ip) => ip.to_string(),
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        if mapping.contains_key(&internal_ip) {
            if request.overwrite_existing {
                mapping.insert(internal_ip, elastic_ip);
                updated += 1;
            } else {
                skipped += 1;
            }
        } else {
            mapping.insert(internal_ip, elastic_ip);
            added += 1;
        }
    }

    save_elastic_ip_mapping_internal(&mapping)?;

    Ok(BulkElasticIpResult {
        added,
        updated,
        skipped,
    })
}

#[tauri::command]
pub fn get_elastic_ip_mapping(internal_ip: String) -> Result<Option<String>, String> {
    let mapping = load_elastic_ip_mapping_internal()?;
    let normalized = parse_ipv4(&internal_ip)?.to_string();
    Ok(mapping.get(&normalized).cloned())
}

#[tauri::command]
pub fn get_all_elastic_mappings() -> Result<Vec<ElasticMappingEntry>, String> {
    let mapping = load_elastic_ip_mapping_internal()?;
    let mut entries = mapping
        .into_iter()
        .map(|(internal_ip, elastic_ip)| ElasticMappingEntry {
            internal_ip,
            elastic_ip,
        })
        .collect::<Vec<_>>();

    entries.sort_by(|a, b| a.internal_ip.cmp(&b.internal_ip));
    Ok(entries)
}

#[tauri::command]
pub fn remove_elastic_ip_mapping(internal_ip: String) -> Result<(), String> {
    let (mut mapping, _) = build_elastic_entry_map()?;
    let normalized = parse_ipv4(&internal_ip)?.to_string();
    mapping.remove(&normalized);
    save_elastic_ip_mapping_internal(&mapping)
}

#[tauri::command]
pub fn get_next_available_elastic_ip(
    request: NextAvailableElasticIpRequest,
) -> Result<NextAvailableElasticIpResponse, String> {
    let mapping = load_elastic_ip_mapping_internal()?;
    let used: HashSet<Ipv4Addr> = mapping
        .values()
        .filter_map(|value| value.parse::<Ipv4Addr>().ok())
        .collect();

    let start = parse_ipv4(&request.start_ip)?;
    let scan_size = request.scan_size.unwrap_or(DEFAULT_SCAN_SIZE);

    let start_u32 = u32::from(start);
    for offset in 0..scan_size {
        let candidate = match start_u32.checked_add(offset) {
            Some(value) => Ipv4Addr::from(value),
            None => break,
        };
        if !used.contains(&candidate) {
            return Ok(NextAvailableElasticIpResponse {
                next_ip: Some(candidate.to_string()),
            });
        }
    }

    Ok(NextAvailableElasticIpResponse { next_ip: None })
}

#[tauri::command]
pub fn get_isp_list() -> Result<Vec<String>, String> {
    let data = load_isp_data_internal()?;
    let mut keys = data.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    Ok(keys)
}

#[tauri::command]
pub fn get_isp_data() -> Result<HashMap<String, Vec<String>>, String> {
    load_isp_data_internal()
}

#[tauri::command]
pub fn update_isp_data(request: UpdateIspDataRequest) -> Result<IspUpdateResult, String> {
    save_isp_data_internal(&request.data)?;
    let summary = summarize_isp_data(
        &request.data,
        resolve_paths()?.isp_data_file.display().to_string(),
    );
    Ok(summary)
}

#[tauri::command]
pub fn get_isp_summary() -> Result<IspUpdateResult, String> {
    let data = load_isp_data_internal()?;
    let saved_path = resolve_paths()?.isp_data_file.display().to_string();
    Ok(summarize_isp_data(&data, saved_path))
}

#[tauri::command]
pub async fn update_isp_from_github() -> Result<IspUpdateResult, String> {
    let response = reqwest::get(GITHUB_ZIP_URL)
        .await
        .map_err(|err| format!("下载运营商数据失败: {err}"))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|err| format!("读取远程数据失败: {err}"))?;

    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|err| format!("解析压缩包失败: {err}"))?;

    let mut data: HashMap<String, Vec<String>> = HashMap::new();

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|err| format!("读取压缩包条目失败: {err}"))?;

        if !file.name().ends_with(".txt") {
            continue;
        }

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|err| format!("读取数据文件失败: {err}"))?;

        let ip_ranges = content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        let lower_name = file.name().to_lowercase();
        let isp_key = if lower_name.contains("chinanet") {
            "DX"
        } else if lower_name.contains("unicom") {
            "LT"
        } else if lower_name.contains("cmcc") {
            "YD"
        } else {
            "OTHER"
        };

        data.entry(isp_key.to_string())
            .or_default()
            .extend(ip_ranges);
    }

    save_isp_data_internal(&data)?;
    let summary = summarize_isp_data(&data, resolve_paths()?.isp_data_file.display().to_string());
    Ok(summary)
}

#[tauri::command]
pub fn detect_isp_info(ip: String) -> Result<DetectIspResponse, String> {
    let ipv4 = parse_ipv4(&ip)?;
    let data = load_isp_data_internal()?;

    let result = find_isp_for_ip(&ipv4, &data);
    Ok(match result {
        Some((isp, range)) => DetectIspResponse {
            isp: Some(isp),
            matched_range: Some(range),
        },
        None => DetectIspResponse {
            isp: None,
            matched_range: None,
        },
    })
}
