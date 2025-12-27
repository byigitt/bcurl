# bcurl

A blazingly fast, minimal HTTP client in Rust. **Smaller than curl itself.**

## Highlights

- **504 KB** binary - 9% smaller than system curl (553 KB)
- **Same performance** as curl
- **Zero async runtime** - uses synchronous ureq
- **Native TLS** - uses OS TLS (SChannel on Windows)
- **No argument parser dependency** - custom minimal parser

## Features

- HTTP methods: GET, POST, PUT, DELETE, HEAD, PATCH
- Custom headers support (`-H`)
- Request body/data (`-d`)
- Follow redirects (`-L`)
- Timeout configuration (`-m`)
- Output to file (`-o`)
- Include response headers (`-i`)
- HEAD request (`-I`)
- Verbose mode (`-v`)
- Silent mode (`-s`)

## Installation

```bash
cargo build --release
```

The optimized binary will be at `target/release/bcurl.exe` (Windows) or `target/release/bcurl` (Linux/macOS)

## Usage

```bash
# Simple GET request
bcurl https://example.com

# POST with JSON data
bcurl -X POST -d '{"key": "value"}' -H "Content-Type: application/json" https://api.example.com

# Save output to file
bcurl -o output.html https://example.com

# Include response headers
bcurl -i https://example.com

# HEAD request (headers only)
bcurl -I https://example.com

# Verbose mode
bcurl -v https://example.com

# Custom timeout (seconds)
bcurl -m 60 https://example.com

# Multiple headers
bcurl -H "Accept: application/json" -H "Authorization: Bearer token" https://api.example.com

# Silent mode (suppress errors)
bcurl -s https://example.com
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
| `-h` | `--help` | Show help |
| `-V` | `--version` | Show version |

## Benchmarks

### Binary Size Comparison

| Binary | Size | Notes |
|--------|-----:|-------|
| **bcurl** | **504 KB** | Rust + ureq + native-tls |
| curl (Windows) | 553 KB | C + OpenSSL/SChannel |

**bcurl is 9% smaller than system curl!**

### Size Optimization Journey

| Version | Size | Reduction |
|---------|-----:|----------:|
| Original (reqwest + clap) | 2.5 MB | baseline |
| After ureq + rustls | 1.4 MB | -44% |
| After ureq + native-tls | 504 KB | **-80%** |

### Performance Comparison

Tested against Windows curl with httpbin.org:

| Test | bcurl | curl | Result |
|------|------:|-----:|--------|
| GET request | ~700ms | ~700ms | Equal |
| POST request | ~700ms | ~700ms | Equal |
| HEAD request | ~670ms | ~650ms | Equal |

Performance is **network-bound** - both tools are equally fast.

## Architecture

### Why bcurl is small

1. **ureq instead of reqwest**
   - No async runtime (tokio)
   - No hyper HTTP library
   - Minimal dependencies

2. **native-tls instead of rustls**
   - Uses OS TLS implementation (SChannel on Windows)
   - No bundled crypto libraries
   - Smaller binary, same security

3. **Custom argument parser**
   - No clap dependency (~200KB savings)
   - Zero-allocation parsing
   - Handles all curl-compatible flags

4. **Aggressive release optimizations**
   ```toml
   [profile.release]
   lto = "fat"          # Maximum LTO
   codegen-units = 1    # Best optimization
   panic = "abort"      # No unwinding
   strip = true         # Remove symbols
   opt-level = "z"      # Size optimization
   ```

## Library Usage

bcurl can also be used as a library:

```rust
use bcurl::{MinimalCurl, RequestConfig, HttpMethod};
use std::time::Duration;

fn main() -> Result<(), bcurl::CurlError> {
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

## Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check binary size
ls -la target/release/bcurl.exe
```

## Running Benchmarks

```powershell
# Quick benchmark
powershell -ExecutionPolicy Bypass -File benchmark.ps1

# Extended benchmark suite
powershell -ExecutionPolicy Bypass -File benchmark-extended.ps1
```

## Why "bcurl"?

- **b** = blazingly fast, binary-optimized, better
- **curl** = the tool we all know and love

bcurl aims to be a minimal, fast, drop-in replacement for common curl use cases.

## License

MIT
