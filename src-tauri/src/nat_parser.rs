use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NatEntry {
    pub name: String,
    pub protocol: String,
    pub global_ip: String,
    pub global_port: String,
    pub inside_ip: String,
    pub inside_port: String,
    pub vrrp: Option<String>,
    pub rule: Option<String>,
    pub description: Option<String>,
    pub command: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NatParseResult {
    pub success_entries: Vec<NatEntry>,
    pub failed_entries: Vec<String>,
    pub device_type: String,
}

pub struct NATParser;

impl NATParser {
    pub fn parse_config(text: &str, device_type: &str) -> NatParseResult {
        let mut success_entries = Vec::new();
        let mut failed_entries = Vec::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with("nat server") {
                match device_type {
                    "huawei" => {
                        if let Some(parsed) = Self::parse_huawei_command(line) {
                            success_entries.push(parsed);
                        } else {
                            failed_entries.push(line.to_string());
                        }
                    }
                    "h3c" => {
                        if let Some(parsed) = Self::parse_h3c_command(line) {
                            success_entries.push(parsed);
                        } else {
                            failed_entries.push(line.to_string());
                        }
                    }
                    _ => {
                        failed_entries.push(line.to_string());
                    }
                }
            }
        }

        NatParseResult {
            success_entries,
            failed_entries,
            device_type: device_type.to_string(),
        }
    }

    fn parse_huawei_command(command: &str) -> Option<NatEntry> {
        let (processed, quoted_names) = preprocess_command(command);
        let tokens: Vec<&str> = processed.split_whitespace().collect();
        if tokens.len() < 2 || tokens[0] != "nat" || tokens[1] != "server" {
            return None;
        }

        let mut idx = 2;
        let mut name: Option<String> = None;

        while idx < tokens.len() && !is_huawei_keyword(tokens[idx]) {
            let token = restore_token(tokens[idx], &quoted_names);
            if name.is_none() {
                name = Some(token);
            }
            idx += 1;
        }

        let mut protocol = "any".to_string();
        let mut global_ip = String::new();
        let mut global_port = "any".to_string();
        let mut inside_ip = String::new();
        let mut inside_port = "any".to_string();

        while idx < tokens.len() {
            match tokens[idx] {
                "protocol" => {
                    if idx + 1 < tokens.len() {
                        protocol = restore_token(tokens[idx + 1], &quoted_names);
                        idx += 2;
                    } else {
                        idx += 1;
                    }
                }
                "global" => {
                    if idx + 1 < tokens.len() {
                        global_ip = restore_token(tokens[idx + 1], &quoted_names);
                        idx += 2;

                        if idx < tokens.len() && !is_huawei_keyword(tokens[idx]) {
                            let first_port = restore_token(tokens[idx], &quoted_names);
                            idx += 1;

                            let second_port = if idx < tokens.len()
                                && !is_huawei_keyword(tokens[idx])
                                && is_port_number(tokens[idx])
                                && is_port_number(&first_port)
                            {
                                let port = restore_token(tokens[idx], &quoted_names);
                                idx += 1;
                                Some(port)
                            } else {
                                None
                            };

                            global_port = match second_port {
                                Some(end) => format!("{}-{}", first_port, end),
                                None => first_port,
                            };
                        }
                    } else {
                        idx += 1;
                    }
                }
                "inside" => {
                    if idx + 1 < tokens.len() {
                        inside_ip = restore_token(tokens[idx + 1], &quoted_names);
                        idx += 2;

                        if idx < tokens.len() && !is_huawei_keyword(tokens[idx]) {
                            let first_port = restore_token(tokens[idx], &quoted_names);
                            idx += 1;

                            let second_port = if idx < tokens.len()
                                && !is_huawei_keyword(tokens[idx])
                                && is_port_number(tokens[idx])
                                && is_port_number(&first_port)
                            {
                                let port = restore_token(tokens[idx], &quoted_names);
                                idx += 1;
                                Some(port)
                            } else {
                                None
                            };

                            inside_port = match second_port {
                                Some(end) => format!("{}-{}", first_port, end),
                                None => first_port,
                            };
                        }
                    } else {
                        idx += 1;
                    }
                }
                "zone" | "acl" => {
                    idx += 1;
                    if idx < tokens.len() && !is_huawei_keyword(tokens[idx]) {
                        idx += 1;
                    }
                }
                "no-reverse" | "reversible" | "unr-route" => {
                    idx += 1;
                }
                _ => {
                    idx += 1;
                }
            }
        }

        if global_ip.is_empty() || inside_ip.is_empty() {
            return None;
        }

        let name = name.unwrap_or_else(|| {
            let display_global_port = if global_port.is_empty() {
                "any".to_string()
            } else {
                global_port.clone()
            };
            let display_inside_port = if inside_port.is_empty() {
                "any".to_string()
            } else {
                inside_port.clone()
            };
            format!(
                "{} {}:{} -> {}:{}",
                protocol, global_ip, display_global_port, inside_ip, display_inside_port
            )
        });

        Some(NatEntry {
            name,
            protocol,
            global_ip,
            global_port,
            inside_ip,
            inside_port,
            vrrp: None,
            rule: None,
            description: None,
            command: command.to_string(),
        })
    }

    fn parse_h3c_command(command: &str) -> Option<NatEntry> {
        let protocol =
            capture_token(h3c_protocol_re(), command).unwrap_or_else(|| "any".to_string());
        let global_ip = capture_token(h3c_global_re(), command)?;
        let inside_ip = capture_token(h3c_inside_re(), command)?;
        let rule = capture_token(h3c_rule_re(), command).unwrap_or_else(|| "-".to_string());
        let vrrp = capture_token(h3c_vrrp_re(), command).unwrap_or_else(|| "-".to_string());

        let description = capture_token(h3c_description_re(), command)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "-".to_string());

        let mut global_port = "any".to_string();
        let mut inside_port = "any".to_string();

        if protocol.to_lowercase() != "any" {
            if let Some(caps) = h3c_double_port_re().captures(command) {
                global_port = format!("{}-{}", &caps[1], &caps[2]);
                inside_port = format!("{}-{}", &caps[3], &caps[4]);
            } else if let Some(caps) = h3c_single_port_re().captures(command) {
                global_port = caps[1].to_string();
                inside_port = caps[2].to_string();
            }
        }

        Some(NatEntry {
            name: rule.clone(),
            protocol,
            global_ip,
            global_port,
            inside_ip,
            inside_port,
            vrrp: Some(vrrp),
            rule: Some(rule),
            description: Some(description),
            command: command.to_string(),
        })
    }
}

#[tauri::command]
pub fn parse_nat_config(text: String, device_type: String) -> NatParseResult {
    NATParser::parse_config(&text, &device_type)
}

fn preprocess_command(command: &str) -> (String, Vec<String>) {
    let mut processed = command.to_string();
    let mut quoted_values = Vec::new();

    for (index, caps) in quoted_value_re().captures_iter(command).enumerate() {
        let quoted = caps[1].to_string();
        quoted_values.push(quoted.clone());
        let placeholder = format!("QUOTED_NAME_{}", index);
        processed = processed.replace(&format!("\"{}\"", quoted), &placeholder);
    }

    (processed, quoted_values)
}

fn restore_token(token: &str, quoted_values: &[String]) -> String {
    if let Some(rest) = token.strip_prefix("QUOTED_NAME_") {
        if let Ok(index) = rest.parse::<usize>() {
            if let Some(value) = quoted_values.get(index) {
                return value.clone();
            }
        }
    }
    token.to_string()
}

fn capture_token(regex: &Regex, command: &str) -> Option<String> {
    regex
        .captures(command)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

fn quoted_value_re() -> &'static Regex {
    static QUOTED_RE: OnceLock<Regex> = OnceLock::new();
    QUOTED_RE.get_or_init(|| Regex::new(r#""([^"]+)""#).unwrap())
}

fn h3c_protocol_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\bprotocol\s+(\S+)").unwrap())
}

fn h3c_global_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\bglobal\s+(\S+)").unwrap())
}

fn h3c_inside_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\binside\s+(\S+)").unwrap())
}

fn h3c_rule_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\brule\s+(\S+)").unwrap())
}

fn h3c_vrrp_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\bvrrp\s+(\d+)").unwrap())
}

fn h3c_description_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"description\s+(.+?)(?:\s+counting|\s+reversible|$)").unwrap())
}

fn h3c_double_port_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"global\s+\S+\s+(\d+)\s+(\d+)\s+inside\s+\S+\s+(\d+)\s+(\d+)").unwrap()
    })
}

fn h3c_single_port_re() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"global\s+\S+\s+(\d+)\s+inside\s+\S+\s+(\d+)").unwrap())
}

fn is_huawei_keyword(token: &str) -> bool {
    matches!(
        token,
        "protocol"
            | "global"
            | "inside"
            | "zone"
            | "acl"
            | "no-reverse"
            | "reversible"
            | "unr-route"
            | "vpn-instance"
    )
}

fn is_port_number(token: &str) -> bool {
    token.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_huawei_with_protocol_and_ports() {
        let command = r#"nat server "web service" policy1 protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080"#;
        let entry = NATParser::parse_huawei_command(command).expect("should parse");

        assert_eq!(entry.name, "web service");
        assert_eq!(entry.protocol, "tcp");
        assert_eq!(entry.global_ip, "202.100.10.1");
        assert_eq!(entry.global_port, "80");
        assert_eq!(entry.inside_ip, "192.168.1.100");
        assert_eq!(entry.inside_port, "8080");
    }

    #[test]
    fn parse_huawei_simple_command() {
        let command = "nat server web global 10.0.0.1 inside 192.168.1.2";
        let entry = NATParser::parse_huawei_command(command).expect("should parse");

        assert_eq!(entry.name, "web");
        assert_eq!(entry.protocol, "any");
        assert_eq!(entry.global_port, "any");
        assert_eq!(entry.inside_port, "any");
    }

    #[test]
    fn parse_huawei_without_name() {
        let command =
            "nat server protocol tcp global 202.100.10.1 80 inside 192.168.1.100 8080 no-reverse";
        let entry = NATParser::parse_huawei_command(command).expect("should parse");

        assert_eq!(entry.protocol, "tcp");
        assert_eq!(entry.global_ip, "202.100.10.1");
        assert_eq!(entry.global_port, "80");
        assert_eq!(entry.inside_ip, "192.168.1.100");
        assert_eq!(entry.inside_port, "8080");
    }

    #[test]
    fn parse_huawei_udp_range() {
        let command = "nat server YD_UDP10.157.50.178:13102-13109 protocol udp global 183.215.36.4 13102 13109 inside 172.30.208.198 13102 13109 no-reverse";
        let entry = NATParser::parse_huawei_command(command).expect("should parse");

        assert_eq!(entry.protocol, "udp");
        assert_eq!(entry.global_port, "13102-13109");
        assert_eq!(entry.inside_port, "13102-13109");
    }

    #[test]
    fn parse_h3c_with_ports_and_description() {
        let command = "nat server protocol tcp global 203.0.113.1 80 inside 10.0.0.5 8080 rule 100 vrrp 1 description Web Service counting";
        let entry = NATParser::parse_h3c_command(command).expect("should parse");

        assert_eq!(entry.protocol, "tcp");
        assert_eq!(entry.global_ip, "203.0.113.1");
        assert_eq!(entry.global_port, "80");
        assert_eq!(entry.inside_ip, "10.0.0.5");
        assert_eq!(entry.inside_port, "8080");
        assert_eq!(entry.rule.as_deref(), Some("100"));
        assert_eq!(entry.vrrp.as_deref(), Some("1"));
        assert_eq!(entry.description.as_deref(), Some("Web Service"));
    }
}
