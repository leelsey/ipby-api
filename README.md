# IPby API

## Usage

#### Service

```sh
curl https://api.ipby.net
```

#### Self-hosting

```sh
ipby-server --ip 0.0.0.0 --port 3000
```

```sh
curl http://SERVER_IP:3000
```

### Routing

You can use it by accessing [api.ipby.net](https://api.ipby.net). You can use each route as follows.
| Path | Method | Description |
| --- | --- | --- |
| / | GET/POST | Returns the client's IP as text |
| /ip | GET/POST | Returns the client's IP version 4 or 6 as text |
| /ipv4 | GET/POST | Returns the client's IP version 4 as text |
| /ipv6 | GET/POST | Returns the client's IP version 6 as text |
| /xff | GET/POST | Returns the client's `X-Forwarded-For` header |
| /:format | GET/POST | Returns the client's IP as each format |
| /:format/ip | GET/POST | Returns the client's IP version 4 or 6 as each format |
| /:format/ipv4 | GET/POST | Returns the client's IP version 4 as each format |
| /:format/ipv6 | GET/POST | Returns the client's IP version 6 as each format |

### Supported Format

| Format | MIME             |
| ------ | ---------------- |
| text   | text/plain       |
| json   | application/json |
| yaml   | application/yaml |
| toml   | application/toml |
| xml    | application/xml  |

## Build

### 1) Build

It is served by own server.

```sh
cargo build --release --bin ipby-server --features ipby-server
```

This is _CURL_ for testing.

```sh
curl -H "x-forwarded-for: 9.9.9.9" localhost:3000/ip
```

### 2) AWS

When you run this, you need [`cargo-lambda`](https://github.com/awslabs/aws-lambda-rust-runtime). And default build target is `--arm64` for Amazon Linux 2023 (arm64).

#### 2-1) Lambda

It is served by Lambda’s Function URL.

```sh
cargo lambda build --release --arm64 --output-format zip --bin aws-lambda --features aws-lambda
```

This is _AWS Lambda’s Test Event_ for testing.

```json
{
  "resource": "/{proxy+}",
  "path": "/ip",
  "httpMethod": "GET",
  "headers": {
    "x-forwarded-for": "9.9.9.9"
  },
  "requestContext": {
    "resourcePath": "/{proxy+}",
    "httpMethod": "GET",
    "path": "/ip"
  },
  "body": null,
  "isBase64Encoded": false
}
```

#### 2-2) AWS Lambda with API Gateway (HTTP API)

> [!WARNING]
> API Gateway does not currently support IP version 6.

This is serviced by linking Lambda and API Gateway.

```sh
cargo lambda build --release --arm64 --output-format zip --bin aws-api --features aws-api
```

This is _AWS Lambda’s Test Event_ for testing.

```json
{
  "path": "/ip",
  "httpMethod": "GET",
  "headers": {
    "Accept": "*/*",
    "x-forwarded-for": "9.9.9.9"
  },
  "queryStringParameters": null,
  "body": null,
  "isBase64Encoded": false
}
```

## Development

If you use _rust-analyzer_ and have `proc macro main not expanded: No proc-macros present for craterust-analyzerunresolved-proc-macro` error, you need disable it.

This is _VSCode’s Settings_ for disabling it.

```json
{
  "rust-analyzer.procMacro.enable": false
}
```
