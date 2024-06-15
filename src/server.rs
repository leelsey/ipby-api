use axum::response::Response;
use axum::{
    body::Body,
    extract::{Path, Query},
    http::HeaderMap,
    routing::get,
    Router,
};
use clap::Parser;
use serde_json::Value;
use std::collections::HashMap;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
mod ip_response;
use ip_response::{
    extract_ips, get_ip, ip_json_response, ip_jsonp_response, ip_text_response, ip_toml_response,
    ip_xml_response, ip_yaml_response, is_ipv4, is_ipv6,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0")]
    ip: String,
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        r#"
   _______  __
  /  _/ _ \/ /  __ __
 _/ // ___/ _ \/ // /
/___/_/  /_.__/\_, /
              /___/
    IPby API v{}
"#,
        version
    );
    let args = Args::parse();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("Starting server");
    let bind_address = format!("{}:{}", args.ip, args.port);
    info!("Running server on {}", bind_address);
    let app = Router::new().route("/*path", get(handle_request));
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(handle_shutdown())
        .await
        .unwrap();
}

async fn handle_request(
    Path(path): Path<String>,
    headers: HeaderMap,
    Query(params_query): Query<HashMap<String, String>>,
) -> Response<Body> {
    let trusted_proxies = vec![""]; // e.g. "127.0.0.1", "10.10.10.10"
    let x_forwarded_for = headers
        .get("x-forwarded-for")
        .and_then(|v: &axum::http::HeaderValue| v.to_str().ok())
        .unwrap_or("");
    let source_ip = headers
        .get("source-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let client_ip = get_ip(x_forwarded_for, source_ip, &trusted_proxies);
    info!("Received request from {} ({})", client_ip, x_forwarded_for);
    const FORMATS: &[&str] = &["json", "jsonp", "xml", "yaml", "toml"];
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let (format, sub_path) = match path_segments.as_slice() {
        [] => (Some(""), Some("")),
        [format] => (Some(*format), Some("")),
        [format, sub_path] => (Some(*format), Some(*sub_path)),
        _ => (None, None),
    };
    let response = match (format, sub_path) {
        (Some(""), Some("")) => ip_response(Some(client_ip), None, "", true, &params_query),
        (Some("ip"), Some("")) => {
            let (ipv4, ipv6) = extract_ips(client_ip);
            ip_response(ipv4, ipv6, "", false, &params_query)
        }
        (Some("ipv4"), Some("")) if is_ipv4(client_ip) => {
            ip_response(Some(client_ip), None, "", false, &params_query)
        }
        (Some("ipv4"), Some("")) => response_403("IPv4 only"),
        (Some("ipv6"), Some("")) if is_ipv6(client_ip) => {
            ip_response(None, Some(client_ip), "", false, &params_query)
        }
        (Some("ipv6"), Some("")) => response_403("IPv6 only"),
        (Some("xff"), Some("")) => response_200("text/plain", x_forwarded_for),
        (Some(fmt), Some("")) if FORMATS.contains(&fmt) => {
            ip_response(Some(client_ip), Some(client_ip), fmt, true, &params_query)
        }
        (Some(fmt), Some("ip")) if FORMATS.contains(&fmt) => {
            let (ipv4, ipv6) = extract_ips(client_ip);
            ip_response(ipv4, ipv6, fmt, false, &params_query)
        }
        (Some(fmt), Some("ipv4")) if FORMATS.contains(&fmt) && is_ipv4(client_ip) => {
            ip_response(Some(client_ip), None, fmt, false, &params_query)
        }
        (Some(fmt), Some("ipv4")) if FORMATS.contains(&fmt) => response_403("IPv4 only"),
        (Some(fmt), Some("ipv6")) if FORMATS.contains(&fmt) && is_ipv6(client_ip) => {
            ip_response(None, Some(client_ip), fmt, false, &params_query)
        }
        (Some(fmt), Some("ipv6")) if FORMATS.contains(&fmt) => response_403("IPv6 only"),
        _ => response_404(),
    };
    response
}

async fn handle_shutdown() {
    let _ = signal::ctrl_c().await;
    info!("Shutting down");
}

fn response_200(content_type: &str, response_body: &str) -> Response<Body> {
    Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .body(Body::from(response_body.to_string()))
        .unwrap()
}

fn response_403(forbidden_reason: &str) -> Response<Body> {
    Response::builder()
        .status(403)
        .header("Content-Type", "text/plain")
        .body(Body::from(format!("Forbidden: {}", forbidden_reason)))
        .unwrap()
}

fn response_404() -> Response<Body> {
    Response::builder()
        .status(404)
        .header("Content-Type", "text/plain")
        .body(Body::from("Not Found"))
        .unwrap()
}

fn ip_response(
    ipv4: Option<&str>,
    ipv6: Option<&str>,
    format_type: &str,
    check_ipv: bool,
    params_query: &HashMap<String, String>,
) -> Response<Body> {
    let params_json: serde_json::Map<String, Value> = params_query
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect();
    let (response_body, content_type) = match format_type {
        "json" => (
            ip_json_response(ipv4, ipv6, check_ipv).to_string(),
            "application/json",
        ),
        "jsonp" => (
            ip_jsonp_response(ipv4, ipv6, check_ipv, &params_json),
            "application/javascript",
        ),
        "yaml" => (ip_yaml_response(ipv4, ipv6, check_ipv), "application/yaml"),
        "toml" => (ip_toml_response(ipv4, ipv6, check_ipv), "application/toml"),
        "xml" => (ip_xml_response(ipv4, ipv6, check_ipv), "application/xml"),
        _ => (ip_text_response(ipv4, ipv6, check_ipv), "text/plain"),
    };
    response_200(content_type, &response_body)
}
