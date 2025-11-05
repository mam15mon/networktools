use num_bigint::BigUint;
use num_traits::{One, Zero};
use serde::Serialize;
use std::net::Ipv6Addr;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ipv6SubnetResult {
    pub ipv6_address: String,
    pub full_ipv6_address: String,
    pub total_ip_addresses: String,
    pub network_address: String,
    pub network_address_full: String,
    pub ip_range: String,
    pub addresses: Vec<String>,
}

#[tauri::command]
pub fn compute_ipv6_subnet(input: &str) -> Result<Ipv6SubnetResult, String> {
    let normalized = input.trim();
    let (ip_part, cidr_part) = normalized
        .split_once('/')
        .ok_or_else(|| "请输入合法的 IPv6/CIDR，例如 2001:db8::1/64".to_string())?;

    let ipv6_addr: Ipv6Addr = ip_part
        .parse()
        .map_err(|_| "IPv6 地址格式不正确".to_string())?;

    let cidr: u32 = cidr_part
        .parse()
        .map_err(|_| "CIDR 必须是 0 到 128 的整数".to_string())?;

    if cidr > 128 {
        return Err("CIDR 必须在 0 到 128 之间".to_string());
    }

    let ip_big = ipv6_to_biguint(&ipv6_addr);
    let mask_u128: u128 = if cidr == 0 {
        0
    } else {
        u128::MAX << (128 - cidr)
    };
    let mask_big = BigUint::from(mask_u128);
    let network_big = &ip_big & &mask_big;

    let host_bits = 128 - cidr;
    let total_addresses = if host_bits == 0 {
        BigUint::one()
    } else {
        BigUint::one() << host_bits
    };

    let broadcast_big = if total_addresses.is_zero() {
        network_big.clone()
    } else {
        &network_big + (&total_addresses - BigUint::one())
    };

    let network_addr = biguint_to_ipv6(&network_big);
    let broadcast_addr = biguint_to_ipv6(&broadcast_big);

    let display_count = std::cmp::min(total_addresses.clone(), BigUint::from(999u32));
    let mut addresses = Vec::new();
    let mut current = network_big.clone();
    let mut generated = BigUint::zero();
    while generated < display_count {
        addresses.push(format_full_ipv6(&biguint_to_ipv6(&current)));
        current = &current + BigUint::one();
        generated = generated + BigUint::one();
    }

    let total_ip_addresses = format_biguint(&total_addresses);
    let ipv6_address = format!("{}/{}", ipv6_addr, cidr);

    Ok(Ipv6SubnetResult {
        ipv6_address,
        full_ipv6_address: format_full_ipv6(&ipv6_addr),
        total_ip_addresses,
        network_address: network_addr.to_string(),
        network_address_full: format_full_ipv6(&network_addr),
        ip_range: format!(
            "{} - {}",
            format_full_ipv6(&network_addr),
            format_full_ipv6(&broadcast_addr)
        ),
        addresses,
    })
}

fn ipv6_to_biguint(addr: &Ipv6Addr) -> BigUint {
    BigUint::from_bytes_be(&addr.octets())
}

fn biguint_to_ipv6(value: &BigUint) -> Ipv6Addr {
    let mut bytes = [0u8; 16];
    let value_bytes = value.to_bytes_be();
    let offset = 16usize.saturating_sub(value_bytes.len());
    bytes[offset..].copy_from_slice(&value_bytes[value_bytes.len().saturating_sub(16)..]);
    Ipv6Addr::from(bytes)
}

fn format_biguint(value: &BigUint) -> String {
    let mut result = String::new();
    let digits = value.to_str_radix(10);
    let chars: Vec<char> = digits.chars().collect();
    let mut count = 0;
    for ch in chars.iter().rev() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(*ch);
        count += 1;
    }
    result.chars().rev().collect()
}

fn format_full_ipv6(addr: &Ipv6Addr) -> String {
    let segments = addr.segments();
    format!(
        "{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}",
        segments[0],
        segments[1],
        segments[2],
        segments[3],
        segments[4],
        segments[5],
        segments[6],
        segments[7]
    )
}
