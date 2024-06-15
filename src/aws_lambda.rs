use lambda_http::{service_fn, Body, Error, Request, Response};
use serde_json::Value;
use tokio::main;
mod ip_response;
use ip_response::{
    extract_ips, get_ip, ip_json_response, ip_jsonp_response, ip_text_response, ip_toml_response,
    ip_xml_response, ip_yaml_response, is_ipv4, is_ipv6,
};

#[main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(func)).await?;
    Ok(())
}

async fn func(request: Request) -> Result<Response<Body>, Error> {
    let map_null = serde_json::Map::new();
    let event: Value = serde_json::from_slice(request.body().as_ref()).unwrap_or_default();
    let path = request
        .uri()
        .path()
        .trim_start_matches('/')
        .trim_end_matches('/');
    let proxy_path = event
        .get("pathParameters")
        .and_then(|params| params.get("proxy"))
        .and_then(|v| v.as_str())
        .map(|p| p.trim_start_matches('/').trim_end_matches('/'))
        .unwrap_or(""); // API Gateway (HTTP API) `/{proxy+}`
    let valid_path = if proxy_path.is_empty() {
        path
    } else {
        proxy_path
    };
    let main_path = valid_path.split('/').next().unwrap_or("");
    let sub_path = valid_path
        .strip_prefix(&format!("{}/", main_path))
        .unwrap_or("");
    let headers = request.headers();
    let trusted_proxies = vec![""]; // e.g. "127.0.0.1", "10.10.10.10"
    let x_forwarded_for = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let source_ip = headers
        .get("source-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let client_ip = get_ip(x_forwarded_for, source_ip, &trusted_proxies);
    let params_query = event
        .get("queryStringParameters")
        .and_then(|v| v.as_object())
        .unwrap_or(&map_null);
    const FORMATS: &[&str] = &["json", "jsonp", "xml", "yaml", "toml"];
    let response = match (main_path, sub_path) {
        ("", "") => ip_response(Some(client_ip), None, "", true, params_query)?,
        ("ip", "") => {
            let (ipv4, ipv6) = extract_ips(client_ip);
            ip_response(ipv4, ipv6, "", false, params_query)?
        }
        ("ipv4", "") if is_ipv4(client_ip) => {
            ip_response(Some(client_ip), None, "", false, params_query)?
        }
        ("ipv4", "") => response_403("IPv4 only")?,
        ("ipv6", "") if is_ipv6(client_ip) => {
            ip_response(None, Some(client_ip), "", false, params_query)?
        }
        ("ipv6", "") => response_403("IPv6 only")?,
        ("xff", "") => response_200("text/plain", x_forwarded_for)?,
        (fmt, "") if FORMATS.contains(&fmt) => ip_response(
            Some(client_ip),
            Some(client_ip),
            main_path,
            true,
            params_query,
        )?,
        (fmt, "ip") if FORMATS.contains(&fmt) => {
            let (ipv4, ipv6) = extract_ips(client_ip);
            ip_response(ipv4, ipv6, main_path, false, params_query)?
        }
        (fmt, "ipv4") if FORMATS.contains(&fmt) && is_ipv4(client_ip) => {
            ip_response(Some(client_ip), None, main_path, false, params_query)?
        }
        (fmt, "ipv4") if FORMATS.contains(&fmt) => response_403("IPv4 only")?,
        (fmt, "ipv6") if FORMATS.contains(&fmt) && is_ipv6(client_ip) => {
            ip_response(None, Some(client_ip), main_path, false, params_query)?
        }
        (fmt, "ipv6") if FORMATS.contains(&fmt) => response_403("IPv6 only")?,
        _ => response_404()?,
    };
    Ok(response)
}

fn response_200(content_type: &str, response_body: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .body(Body::from(response_body.to_string()))
        .map_err(Box::new)?)
}

fn response_403(forbidden_reason: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(403)
        .header("Content-Type", "text/plain")
        .body(Body::from(format!("Forbidden: {}", forbidden_reason)))
        .map_err(Box::new)?)
}

fn response_404() -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(404)
        .header("Content-Type", "text/plain")
        .body(Body::from("Not Found"))
        .map_err(Box::new)?)
}

fn ip_response(
    ipv4: Option<&str>,
    ipv6: Option<&str>,
    format_type: &str,
    check_ipv: bool,
    params_query: &serde_json::Map<String, Value>,
) -> Result<Response<Body>, Error> {
    let (response_body, content_type) = match format_type {
        "json" => (
            ip_json_response(ipv4, ipv6, check_ipv).to_string(),
            "application/json",
        ),
        "jsonp" => (
            ip_jsonp_response(ipv4, ipv6, check_ipv, params_query),
            "application/javascript",
        ),
        "yaml" => (ip_yaml_response(ipv4, ipv6, check_ipv), "application/yaml"),
        "toml" => (ip_toml_response(ipv4, ipv6, check_ipv), "application/toml"),
        "xml" => (ip_xml_response(ipv4, ipv6, check_ipv), "application/xml"),
        _ => (ip_text_response(ipv4, ipv6, check_ipv), "text/plain"),
    };
    response_200(content_type, &response_body)
}