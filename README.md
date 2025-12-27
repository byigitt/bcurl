# minimal-curl

A minimal, fast HTTP client implementation in Rust. Inspired by curl, built for speed.

## Features

- HTTP methods: GET, POST, PUT, DELETE, HEAD, PATCH
- Custom headers support
- Request body/data
- Follow redirects
- Timeout configuration
- Output to file
- Verbose mode
- Silent mode

## Installation

```bash
cargo build --release
```

The optimized binary will be at `target/release/minimal-curl.exe`

## Usage

```bash
# Simple GET request
minimal-curl https://example.com

# POST with data
minimal-curl -X POST -d '{"key": "value"}' -H "Content-Type: application/json" https://api.example.com

# Save output to file
minimal-curl -o output.html https://example.com

# Include response headers
minimal-curl -i https://example.com

# HEAD request (headers only)
minimal-curl -I https://example.com

# Verbose mode
minimal-curl -v https://example.com

# Custom timeout (seconds)
minimal-curl -m 60 https://example.com

# Multiple headers
minimal-curl -H "Accept: application/json" -H "Authorization: Bearer token" https://api.example.com
```

### Options

| Option | Long | Description |
|--------|------|-------------|
| `-X` | `--request` | HTTP method (GET, POST, PUT, DELETE, HEAD, PATCH) |
| `-d` | `--data` | Request body data |
| `-H` | `--header` | Add header (can be used multiple times) |
| `-o` | `--output` | Write output to file |
| `-i` | `--include` | Include response headers in output |
| `-I` | `--head` | Show headers only (HEAD request) |
| `-L` | `--location` | Follow redirects (default: true) |
| `-v` | `--verbose` | Verbose output |
| `-s` | `--silent` | Silent mode |
| `-m` | `--max-time` | Maximum time in seconds (default: 30) |

## Performance

minimal-curl is optimized for speed and outperforms curl in most benchmarks.

### Benchmark Results

Tested against Windows curl 8.0.1 (Schannel) with 5 iterations per test:

| Test | minimal-curl | curl | Difference |
|------|-------------:|-----:|------------|
| GET Request | 8.0ms | 9.6ms | **17% faster** |
| HEAD Request | 8.0ms | 9.4ms | **15% faster** |
| POST Request | 7.0ms | 8.0ms | **13% faster** |
| PUT Request | 7.0ms | 8.4ms | **17% faster** |
| DELETE Request | 7.2ms | 9.6ms | **25% faster** |
| POST 1KB | 7.8ms | 10.2ms | **24% faster** |
| POST 10KB | 8.8ms | 10.2ms | **14% faster** |
| Response 1KB | 8.6ms | 9.4ms | **9% faster** |
| Response 10KB | 8.0ms | 9.6ms | **17% faster** |
| Response 100KB | 8.0ms | 9.8ms | **18% faster** |
| 5 Custom Headers | 7.0ms | 9.0ms | **22% faster** |
| Cold Start | 705ms | 724ms | **2.5% faster** |

### Performance by Category

| Category | Improvement vs curl |
|----------|--------------------:|
| GET/HEAD requests | 15-17% faster |
| POST/PUT requests | 13-25% faster |
| Response handling | 9-29% faster |
| Header processing | 16-22% faster |
| Cold start | 2.5% faster |

### Binary Size

| Binary | Size |
|--------|-----:|
| minimal-curl | 2.5 MB |
| curl (Windows) | 0.54 MB |

## Optimizations

minimal-curl achieves its performance through several optimizations:

### Build Optimizations

```toml
[profile.release]
lto = true           # Link-Time Optimization
codegen-units = 1    # Single codegen unit for maximum optimization
panic = "abort"      # No unwinding overhead
strip = true         # Strip debug symbols
opt-level = 3        # Maximum optimization
```

### TLS Backend

Uses **rustls** instead of native-tls (Schannel on Windows) for faster TLS initialization:

```toml
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
```

rustls provides:
- Faster cold start (no OS TLS initialization overhead)
- Pure Rust implementation (no FFI overhead)
- Modern TLS 1.3 support

### Minimal Dependencies

Only essential clap features are enabled to reduce binary size and compilation time:

```toml
clap = { version = "4.5", default-features = false, features = ["derive", "std", "help", "usage", "error-context"] }
```

## Library Usage

minimal-curl can also be used as a library:

```rust
use minimal_curl::{MinimalCurl, RequestConfig, HttpMethod};
use std::time::Duration;

fn main() -> Result<(), minimal_curl::CurlError> {
    let client = MinimalCurl::new();

    // Simple GET
    let response = client.get("https://httpbin.org/get")?;
    println!("Status: {}", response.status);
    println!("Body: {}", response.body);

    // POST with configuration
    let config = RequestConfig::new("https://httpbin.org/post")
        .method(HttpMethod::Post)
        .header("Content-Type", "application/json")
        .data(r#"{"hello": "world"}"#)
        .timeout(Duration::from_secs(10));

    let response = client.execute(&config)?;
    assert!(response.is_success());

    Ok(())
}
```

## Running Benchmarks

Benchmark scripts are included to compare performance with curl:

```powershell
# Quick benchmark
powershell -ExecutionPolicy Bypass -File benchmark.ps1

# Extended benchmark suite
powershell -ExecutionPolicy Bypass -File benchmark-extended.ps1
```

## Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

## License

MIT
