use num_format::{Locale, ToFormattedString};
use serde::Serialize;
use std::net::Ipv4Addr;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubnetResult {
    pub ip_address: String,
    pub network_address: String,
    pub broadcast_address: String,
    pub subnet_mask: String,
    pub wildcard_mask: String,
    pub binary_subnet_mask: String,
    pub usable_range: String,
    pub total_hosts: String,
    pub usable_hosts: String,
    pub ip_class: String,
    pub cidr_notation: String,
    pub ip_type: String,
    pub short_notation: String,
    pub binary_id: String,
    pub integer_id: String,
    pub hex_id: String,
    pub in_addr_arpa: String,
    pub ipv4_mapped_address: String,
    pub six_to_four_prefix: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEntry {
    pub network: String,
    pub range: String,
    pub broadcast: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubnetComputation {
    pub result: SubnetResult,
    pub related_networks: Vec<NetworkEntry>,
}

#[tauri::command]
pub async fn get_public_ip() -> Result<String, String> {
    // 使用多个公网 IP 服务作为备选，优先使用国内服务
    let services = vec![
        // 优先使用 ipip.net
        "https://myip.ipip.net/", // ipip.net - 专业 IP 服务
        // 备用纯文本服务
        "https://api.ip.sb/ip",          // IP.SB - 返回纯文本 IP
        "https://api.ipify.org",         // ipify - 全球最大，返回纯文本
        "https://icanhazip.com",         // icanhazip - 返回纯文本
        "https://ipecho.net/plain",      // ipecho - 返回纯文本
        "https://checkip.amazonaws.com", // AWS - 返回纯文本
    ];

    for service in services {
        match reqwest::get(service).await {
            Ok(response) => {
                match response.text().await {
                    Ok(ip_text) => {
                        let ip_text = ip_text.trim();

                        // 特殊处理 ipip.net 的格式
                        if service.contains("ipip.net") {
                            // 从 "当前 IP：111.205.145.129  来自于：中国 北京 北京  联通" 中提取 IP
                            if let Some(ip) = extract_ip_from_ipip_text(ip_text) {
                                return Ok(ip);
                            }
                        } else {
                            // 验证是否是有效的 IPv4 地址
                            if ip_text.parse::<Ipv4Addr>().is_ok() {
                                return Ok(ip_text.to_string());
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
            Err(_) => continue,
        }
    }

    Err("无法获取公网 IP 地址".to_string())
}

// 从 ipip.net 的文本中提取 IP 地址
fn extract_ip_from_ipip_text(text: &str) -> Option<String> {
    // ipip.net 返回格式类似: "当前 IP：111.205.145.129  来自于：中国 北京 北京  联通"

    // 直接使用备用方法：查找 IPv4 地址模式，更安全
    for word in text.split_whitespace() {
        if let Ok(ip) = word.parse::<Ipv4Addr>() {
            return Some(ip.to_string());
        }
    }

    // 如果备用方法失败，尝试正则表达式方法
    use regex::Regex;
    if let Ok(re) = Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b") {
        if let Some(caps) = re.captures(text) {
            if let Some(ip_match) = caps.get(1) {
                if let Ok(ip) = ip_match.as_str().parse::<Ipv4Addr>() {
                    return Some(ip.to_string());
                }
            }
        }
    }

    None
}

#[tauri::command]
pub fn compute_subnet(input: &str) -> Result<SubnetComputation, String> {
    let normalized = input.trim();
    let (ip_part, cidr_part) = normalized
        .split_once('/')
        .ok_or_else(|| "请输入合法的 IPv4/CIDR，例如 192.168.0.1/24".to_string())?;

    let ip_addr: Ipv4Addr = ip_part
        .parse()
        .map_err(|_| "IPv4 地址格式不正确".to_string())?;

    let cidr: u32 = cidr_part
        .parse()
        .map_err(|_| "CIDR 必须是 0 到 32 的整数".to_string())?;

    if cidr > 32 {
        return Err("CIDR 必须在 0 到 32 之间".to_string());
    }

    let ip_int = u32::from(ip_addr);
    let mask = if cidr == 0 {
        0
    } else {
        u32::MAX << (32 - cidr)
    };
    let wildcard = !mask;
    let network = ip_int & mask;
    let broadcast = network | wildcard;

    let total_hosts_value: u128 = if cidr >= 32 { 1 } else { 1u128 << (32 - cidr) };

    let (usable_hosts_value, first_usable, last_usable) = match cidr {
        32 => (1u128, network, network),
        31 => (2u128, network, broadcast),
        _ => {
            let usable = total_hosts_value.saturating_sub(2);
            (
                usable,
                network.saturating_add(1),
                broadcast.saturating_sub(1),
            )
        }
    };

    let usable_range = if cidr == 32 {
        format_ip(first_usable)
    } else {
        format!("{} - {}", format_ip(first_usable), format_ip(last_usable))
    };

    let ip_class = derive_class(ip_int);
    let ip_type = derive_type(ip_int, &ip_class);

    let result = SubnetResult {
        ip_address: ip_addr.to_string(),
        network_address: format_ip(network),
        broadcast_address: format_ip(broadcast),
        subnet_mask: format_ip(mask),
        wildcard_mask: format_ip(wildcard),
        binary_subnet_mask: format_binary(mask, true),
        usable_range,
        total_hosts: total_hosts_value.to_formatted_string(&Locale::en),
        usable_hosts: usable_hosts_value.to_formatted_string(&Locale::en),
        ip_class: ip_class.clone(),
        cidr_notation: format!("/{}", cidr),
        ip_type,
        short_notation: format!("{}/{}", ip_addr, cidr),
        binary_id: format_binary(ip_int, false),
        integer_id: ip_int.to_string(),
        hex_id: format!("0x{:08x}", ip_int),
        in_addr_arpa: build_in_addr_arpa(ip_addr),
        ipv4_mapped_address: build_ipv4_mapped(ip_int),
        six_to_four_prefix: build_six_to_four_prefix(ip_int),
    };

    let related_networks = build_related_networks(ip_int, cidr, &ip_class);

    Ok(SubnetComputation {
        result,
        related_networks,
    })
}

fn build_related_networks(ip_int: u32, cidr: u32, ip_class: &str) -> Vec<NetworkEntry> {
    let class_boundary = match class_boundary(ip_class) {
        Some(boundary) => boundary,
        None => return Vec::new(),
    };

    if cidr < class_boundary {
        return Vec::new();
    }

    let class_mask = if class_boundary == 0 {
        0
    } else {
        u32::MAX << (32 - class_boundary)
    };
    let base_network = ip_int & class_mask;

    let diff = cidr - class_boundary;
    let networks_count = if diff >= 32 { u32::MAX } else { 1u32 << diff };

    let step = if cidr == 32 { 1 } else { 1u32 << (32 - cidr) };

    (0..networks_count.min(256))
        .map(|index| {
            let network_int = base_network.saturating_add(step.saturating_mul(index));
            let broadcast_int = network_int.saturating_add(step.saturating_sub(1));

            let (first_host, last_host) = match cidr {
                32 => (network_int, network_int),
                31 => (network_int, broadcast_int),
                _ => (
                    network_int.saturating_add(1),
                    broadcast_int.saturating_sub(1),
                ),
            };

            NetworkEntry {
                network: format_ip(network_int),
                range: if cidr == 32 {
                    format_ip(first_host)
                } else {
                    format!("{} - {}", format_ip(first_host), format_ip(last_host))
                },
                broadcast: format_ip(broadcast_int),
            }
        })
        .collect()
}

fn format_ip(value: u32) -> String {
    Ipv4Addr::from(value).to_string()
}

fn format_binary(value: u32, with_dots: bool) -> String {
    let binary = format!("{:032b}", value);
    if !with_dots {
        binary
    } else {
        binary
            .as_bytes()
            .chunks(8)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
            .collect::<Vec<_>>()
            .join(".")
    }
}

fn derive_class(ip_int: u32) -> String {
    let first_octet = (ip_int >> 24) as u8;
    if first_octet <= 127 {
        "A".into()
    } else if first_octet <= 191 {
        "B".into()
    } else if first_octet <= 223 {
        "C".into()
    } else if first_octet <= 239 {
        "D".into()
    } else {
        "E".into()
    }
}

fn class_boundary(ip_class: &str) -> Option<u32> {
    match ip_class {
        "A" => Some(8),
        "B" => Some(16),
        "C" => Some(24),
        _ => None,
    }
}

fn derive_type(ip_int: u32, ip_class: &str) -> String {
    let first = (ip_int >> 24) as u8;
    let second = ((ip_int >> 16) & 0xff) as u8;

	let is_private = first == 10
		|| (first == 172 && (16..=31).contains(&second))
		|| (first == 192 && second == 168);

	if is_private {
		"Private".into()
	} else if first == 169 && second == 254 {
		"Link-local".into()
    } else if first == 127 {
        "Loopback".into()
    } else if first == 100 && (64..=127).contains(&second) {
        "Carrier-grade NAT".into()
    } else if (224..=239).contains(&first) {
        "Multicast".into()
    } else if first >= 240 {
        "Reserved".into()
    } else if ip_class == "A" && first == 0 {
        "Special".into()
    } else {
        "Public".into()
    }
}

fn build_in_addr_arpa(ip_addr: Ipv4Addr) -> String {
    let octets = ip_addr.octets();
    format!(
        "{}.{}.{}.{}.in-addr.arpa",
        octets[3], octets[2], octets[1], octets[0]
    )
}

fn build_ipv4_mapped(ip_int: u32) -> String {
    format!(
        "::ffff:{:04x}.{:04x}",
        (ip_int >> 16) & 0xffff,
        ip_int & 0xffff
    )
}

fn build_six_to_four_prefix(ip_int: u32) -> String {
    format!(
        "2002:{:04x}:{:04x}::/48",
        (ip_int >> 16) & 0xffff,
        ip_int & 0xffff
    )
}
