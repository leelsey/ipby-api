use serde_json::{json, Value};

pub fn is_ipv4(ip: &str) -> bool {
    ip.parse::<std::net::Ipv4Addr>().is_ok()
}

pub fn is_ipv6(ip: &str) -> bool {
    ip.parse::<std::net::Ipv6Addr>().is_ok()
}

pub fn get_ip<'a>(
    x_forwarded_for: &'a str,
    source_ip: &'a str,
    trusted_proxies: &'a [&'a str],
) -> &'a str {
    let ip_list: Vec<&str> = x_forwarded_for.split(',').map(|ip| ip.trim()).collect();
    for real_ip in ip_list.iter().rev() {
        if !trusted_proxies.contains(real_ip) {
            return real_ip;
        }
    }
    source_ip
}

pub fn extract_ips(ip: &str) -> (Option<&str>, Option<&str>) {
    if is_ipv4(ip) {
        (Some(ip), None)
    } else if is_ipv6(ip) {
        (None, Some(ip))
    } else {
        (None, None)
    }
}

pub fn ip_text_response(ipv4: Option<&str>, ipv6: Option<&str>, check_ipv: bool) -> String {
    if check_ipv {
        ipv4.or(ipv6).unwrap_or("").to_string()
    } else {
        match (ipv4, ipv6) {
            (Some(ipv4), Some(ipv6)) => format!("{}\n{}", ipv4, ipv6),
            (Some(ipv4), None) => ipv4.to_string(),
            (None, Some(ipv6)) => ipv6.to_string(),
            _ => "".to_string(),
        }
    }
}

pub fn ip_json_response(ipv4: Option<&str>, ipv6: Option<&str>, check_ipv: bool) -> Value {
    if check_ipv {
        json!({ "ip": ipv4.or(ipv6).unwrap_or("") })
    } else {
        let mut response = serde_json::Map::new();
        if let Some(ipv4) = ipv4 {
            response.insert("ipv4".to_string(), json!(ipv4));
        }
        if let Some(ipv6) = ipv6 {
            response.insert("ipv6".to_string(), json!(ipv6));
        }
        json!(response)
    }
}

pub fn ip_jsonp_response(
    ipv4: Option<&str>,
    ipv6: Option<&str>,
    check_ipv: bool,
    callback_param: &serde_json::Map<String, Value>,
) -> String {
    let callback = callback_param
        .get("callback")
        .and_then(|v| v.as_str())
        .unwrap_or("callback");
    format!("{}({});", callback, ip_json_response(ipv4, ipv6, check_ipv))
}

pub fn ip_yaml_response(ipv4: Option<&str>, ipv6: Option<&str>, check_ipv: bool) -> String {
    if check_ipv {
        format!("ip: {}", ipv4.or(ipv6).unwrap_or(""))
    } else {
        let mut response = String::new();
        if let Some(ipv4) = ipv4 {
            response.push_str(&format!("ipv4: {}\n", ipv4));
        }
        if let Some(ipv6) = ipv6 {
            response.push_str(&format!("ipv6: {}\n", ipv6));
        }
        response.trim().to_string()
    }
}

pub fn ip_toml_response(ipv4: Option<&str>, ipv6: Option<&str>, check_ipv: bool) -> String {
    if check_ipv {
        format!("ip = '{}'", ipv4.or(ipv6).unwrap_or(""))
    } else {
        let mut response = String::new();
        if let Some(ipv4) = ipv4 {
            response.push_str(&format!("ipv4 = '{}'\n", ipv4));
        }
        if let Some(ipv6) = ipv6 {
            response.push_str(&format!("ipv6 = '{}'\n", ipv6));
        }
        response.trim().to_string()
    }
}

pub fn ip_xml_response(ipv4: Option<&str>, ipv6: Option<&str>, check_ipv: bool) -> String {
    if check_ipv {
        format!("<ip>{}</ip>", ipv4.or(ipv6).unwrap_or(""))
    } else {
        let mut response = String::new();
        if let Some(ipv4) = ipv4 {
            response.push_str(&format!("<ipv4>{}</ipv4>", ipv4));
        }
        if let Some(ipv6) = ipv6 {
            response.push_str(&format!("<ipv6>{}</ipv6>", ipv6));
        }
        format!("<ip>{}</ip>", response)
    }
}
