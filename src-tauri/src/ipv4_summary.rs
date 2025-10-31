use serde::Serialize;
use std::net::Ipv4Addr;

#[derive(Clone, Copy)]
struct Range {
    start: u32,
    end: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv4AggregateResult {
    pub normalized_inputs: Vec<String>,
    pub precise_summary: Vec<String>,
    pub approximate_summary: String,
    pub non_precise_summary: NonPreciseSummary,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NonPreciseSummary {
    pub cidr: String,
    pub total_addresses: u64,
    pub input_addresses: u64,
    pub extra_addresses: u64,
    pub extra_percentage: f64,
}

#[tauri::command]
pub fn aggregate_ipv4(items: Vec<String>) -> Result<Ipv4AggregateResult, String> {
    let mut ranges: Vec<Range> = Vec::new();
    let mut normalized_inputs: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for raw in items {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        match parse_entry(trimmed) {
            Ok((range, canonical)) => {
                ranges.push(range);
                normalized_inputs.push(canonical);
            }
            Err(err) => {
                errors.push(format!("{} -> {}", trimmed, err));
            }
        }
    }

    if ranges.is_empty() {
        return Err("没有可汇总的 IPv4 数据".into());
    }

    ranges.sort_by_key(|r| r.start);
    let merged = merge_ranges(ranges);

    let mut precise_summary: Vec<String> = Vec::new();
    for range in &merged {
        precise_summary.extend(range_to_cidrs(range.start, range.end));
    }

    let approximate_summary = minimal_supernet(merged.first().unwrap().start, merged.last().unwrap().end);
    let non_precise_summary = calculate_non_precise_summary(&merged);

    Ok(Ipv4AggregateResult {
        normalized_inputs,
        precise_summary,
        approximate_summary,
        non_precise_summary,
        errors,
    })
}

fn parse_entry(entry: &str) -> Result<(Range, String), String> {
    if entry.contains('/') {
        parse_cidr(entry)
    } else if entry.contains('-') {
        parse_range(entry)
    } else {
        parse_single(entry)
    }
}

fn parse_cidr(entry: &str) -> Result<(Range, String), String> {
    let parts: Vec<&str> = entry.split('/').collect();
    if parts.len() != 2 {
        return Err("CIDR 格式不正确".into());
    }
    let ip: Ipv4Addr = parts[0]
        .trim()
        .parse()
        .map_err(|_| "无效的 IPv4 地址".to_string())?;
    let prefix: u32 = parts[1]
        .trim()
        .parse()
        .map_err(|_| "CIDR 前缀必须是数字".to_string())?;
    if prefix > 32 {
        return Err("CIDR 前缀必须在 0-32 之间".into());
    }

    let start = ipv4_to_u32(ip) & mask_for_prefix(prefix);
    let end = start | !mask_for_prefix(prefix);

    Ok((
        Range { start, end },
        format!("{}/{}", Ipv4Addr::from(start), prefix),
    ))
}

fn parse_range(entry: &str) -> Result<(Range, String), String> {
    let parts: Vec<&str> = entry.split('-').collect();
    if parts.len() != 2 {
        return Err("范围格式不正确".into());
    }

    let start_ip: Ipv4Addr = parts[0]
        .trim()
        .parse()
        .map_err(|_| "无效的起始 IP".to_string())?;
    let start_u32 = ipv4_to_u32(start_ip);

    let end_str = parts[1].trim();
    let end_ip = if end_str.contains('.') {
        end_str
            .parse::<Ipv4Addr>()
            .map_err(|_| "无效的结束 IP".to_string())?
    } else {
        let mut octets = start_ip.octets();
        let value: u8 = end_str
            .parse()
            .map_err(|_| "简化范围的结束值必须是数字".to_string())?;
        octets[3] = value;
        Ipv4Addr::from(octets)
    };

    let end_u32 = ipv4_to_u32(end_ip);
    if end_u32 < start_u32 {
        return Err("结束 IP 必须大于或等于起始 IP".into());
    }

    Ok((
        Range {
            start: start_u32,
            end: end_u32,
        },
        format!("{} - {}", start_ip, end_ip),
    ))
}

fn parse_single(entry: &str) -> Result<(Range, String), String> {
    let ip: Ipv4Addr = entry
        .trim()
        .parse()
        .map_err(|_| "无效的 IPv4 地址".to_string())?;
    let value = ipv4_to_u32(ip);
    Ok((Range { start: value, end: value }, ip.to_string()))
}

fn merge_ranges(ranges: Vec<Range>) -> Vec<Range> {
    if ranges.is_empty() {
        return ranges;
    }

    let mut merged: Vec<Range> = Vec::new();
    let mut current = ranges[0];

    for range in ranges.into_iter().skip(1) {
        if range.start <= current.end.saturating_add(1) {
            current.end = current.end.max(range.end);
        } else {
            merged.push(current);
            current = range;
        }
    }
    merged.push(current);

    merged
}

fn range_to_cidrs(start: u32, end: u32) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = start as u64;
    let end64 = end as u64;

    while current <= end64 {
        let cur_u32 = current as u32;
        let max_trailing = cur_u32.trailing_zeros();
        let mut prefix = if max_trailing == 32 { 0 } else { 32 - max_trailing };
        let mut block_size = block_size_for_prefix(prefix);

        while current + block_size - 1 > end64 {
            prefix += 1;
            block_size >>= 1;
        }

        result.push(format!("{}/{}", Ipv4Addr::from(cur_u32), prefix));
        current += block_size;
    }
    result
}

fn block_size_for_prefix(prefix: u32) -> u64 {
    if prefix >= 32 {
        1
    } else {
        1u64 << (32 - prefix)
    }
}

fn minimal_supernet(start: u32, end: u32) -> String {
    let diff = start ^ end;
    let prefix = diff.leading_zeros();
    let mask = if prefix == 0 {
        0u32
    } else {
        u32::MAX << (32 - prefix)
    };
    let network = start & mask;
    format!("{}/{}", Ipv4Addr::from(network), prefix)
}

fn calculate_non_precise_summary(merged_ranges: &[Range]) -> NonPreciseSummary {
    if merged_ranges.is_empty() {
        panic!("合并后的范围不能为空");
    }

    let start = merged_ranges.first().unwrap().start;
    let end = merged_ranges.last().unwrap().end;

    // 计算最小超网
    let diff = start ^ end;
    let prefix = diff.leading_zeros();
    let mask = if prefix == 0 {
        0u32
    } else {
        u32::MAX << (32 - prefix)
    };
    let network = start & mask;

    // CIDR 表示
    let cidr = format!("{}/{}", Ipv4Addr::from(network), prefix);

    // 计算总地址数（处理各种边界情况）
    let total_addresses = if prefix >= 32 {
        1u128
    } else if prefix == 0 {
        1u128 << 32
    } else {
        1u128 << (32 - prefix)
    };

    // 计算输入地址的实际数量
    let mut input_addresses = 0u128;
    for range in merged_ranges {
        let size = (range.end as u128)
            .saturating_sub(range.start as u128)
            + 1;
        input_addresses += size;
    }

    // 计算额外地址数和百分比
    let extra_addresses = if input_addresses > total_addresses {
        0
    } else {
        total_addresses - input_addresses
    };
    let extra_percentage =
        if total_addresses > 0 {
            (extra_addresses as f64 / total_addresses as f64) * 100.0
        } else {
            0.0
        };

    NonPreciseSummary {
        cidr,
        total_addresses: total_addresses as u64,
        input_addresses: input_addresses as u64,
        extra_addresses: extra_addresses as u64,
        extra_percentage,
    }
}

fn ipv4_to_u32(ip: Ipv4Addr) -> u32 {
    u32::from(ip)
}

fn mask_for_prefix(prefix: u32) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}