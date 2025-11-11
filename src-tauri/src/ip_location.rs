use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpLocationResult {
    pub country: String,
    pub region: String,
    pub city: String,
    pub isp: String,
    pub raw_info: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationInfo {
    pub ip: String,
    pub location: IpLocationResult,
    pub is_ipv4: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub api_endpoint: String,
    pub supports_ipv4: bool,
    pub supports_ipv6: bool,
}

const IP_API_URL: &str = "https://api.mir6.com/api/ip";

#[tauri::command]
pub async fn lookup_ip_location(ip: String) -> Result<LocationInfo, String> {
    let url = format!("{}?ip={}&type=json", IP_API_URL, ip);

    // 判断 IP 类型
    let is_ipv4 = ip.contains('.');

    match reqwest::get(&url).await {
        Ok(response) => match response.json::<Value>().await {
            Ok(json_data) => match parse_json_result(&json_data) {
                Ok(location) => Ok(LocationInfo {
                    ip: ip.clone(),
                    location,
                    is_ipv4,
                }),
                Err(e) => Err(format!("解析 JSON 数据失败: {}", e)),
            },
            Err(e) => Err(format!("获取 JSON 响应失败: {}", e)),
        },
        Err(e) => Err(format!("请求失败: {}", e)),
    }
}

fn parse_json_result(json: &Value) -> Result<IpLocationResult, String> {
    // 提取数据字段
    let country = json["data"]["country"]
        .as_str()
        .unwrap_or("未知")
        .to_string();

    let region = json["data"]["province"]
        .as_str()
        .unwrap_or("未知")
        .to_string();

    let city = json["data"]["city"].as_str().unwrap_or("未知").to_string();

    let isp = json["data"]["isp"].as_str().unwrap_or("未知").to_string();

    // 如果城市未知且省份是直辖市，设为相同
    let final_city = if city == "未知"
        && (region.ends_with("市") || region.ends_with("县") || region.ends_with("区"))
    {
        region.clone()
    } else {
        city
    };

    Ok(IpLocationResult {
        country,
        region,
        city: final_city,
        isp,
        raw_info: json.to_string(),
    })
}

#[tauri::command]
pub async fn debug_ip_query(ip: String) -> Result<String, String> {
    let url = format!("{}?ip={}&type=json", IP_API_URL, ip);

    match reqwest::get(&url).await {
        Ok(response) => match response.json::<Value>().await {
            Ok(json) => match parse_json_result(&json) {
                Ok(location) => Ok(format!(
                    "IP: {}\n国家: {}\n省份: {}\n城市: {}\n运营商: {}\n原始JSON: {}",
                    ip,
                    location.country,
                    location.region,
                    location.city,
                    location.isp,
                    serde_json::to_string_pretty(&json).unwrap_or_default()
                )),
                Err(e) => Err(format!("解析JSON失败: {}", e)),
            },
            Err(e) => Err(format!("获取JSON响应失败: {}", e)),
        },
        Err(e) => Err(format!("请求失败: {}", e)),
    }
}

#[tauri::command]
pub fn get_database_info() -> Result<DatabaseInfo, String> {
    Ok(DatabaseInfo {
        api_endpoint: IP_API_URL.to_string(),
        supports_ipv4: true,
        supports_ipv6: true,
    })
}
