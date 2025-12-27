# Agent Instructions

> **CRITICAL - READ EVERY MESSAGE**
>
> Read this file at the **START OF EVERY MESSAGE**. This ensures you have the latest project context.

---

## Quick Reference

```bash
# Essential Commands
cargo build --release       # Build optimized binary (504 KB)
cargo build                 # Debug build
cargo test                  # Run all tests (unit + integration)
cargo clippy                # Lint code
cargo fmt                   # Format code
cargo doc --open            # Generate and view docs

# Run the binary
./target/release/bcurl https://example.com

# Run benchmarks
powershell -ExecutionPolicy Bypass -File benchmark.ps1
powershell -ExecutionPolicy Bypass -File benchmark-extended.ps1
```

---

## Stack

| Component | Technology |
|-----------|------------|
| Language | Rust (2021 edition) |
| HTTP Client | ureq (synchronous, minimal) |
| TLS | native-tls (OS native: SChannel on Windows) |
| Error Handling | thiserror |
| Testing | mockito + tempfile |
| Binary Size | 504 KB (optimized) |

---

## Project Structure

```
minimal-curl/
├── src/
│   ├── main.rs          # CLI entry point, argument parsing
│   └── lib.rs           # Core HTTP client library
├── tests/
│   └── integration_tests.rs  # Integration tests with mockito
├── Cargo.toml           # Dependencies and release profile
├── benchmark.ps1        # Quick benchmark script
├── benchmark-extended.ps1    # Extended benchmark suite
├── README.md            # User documentation
└── BENCHMARKS.md        # Performance comparison data
```

---

## Code Style

### Error Handling
```rust
// ✅ DO: Use thiserror for custom errors
#[derive(Error, Debug)]
pub enum CurlError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] ureq::Error),
}

// ❌ DON'T: Use generic error types
fn bad() -> Result<(), Box<dyn std::error::Error>> { ... }
```

### Builder Pattern
```rust
// ✅ DO: Use builder pattern with method chaining
let config = RequestConfig::new("https://example.com")
    .method(HttpMethod::Post)
    .header("Content-Type", "application/json")
    .data(r#"{"key": "value"}"#)
    .timeout(Duration::from_secs(10));

// ❌ DON'T: Create structs with many constructor args
let config = RequestConfig::new(url, method, headers, data, timeout);
```

### Inline Hints
```rust
// ✅ DO: Use #[inline] for small, frequently-called methods
#[inline]
pub fn is_success(&self) -> bool {
    (200..300).contains(&self.status)
}

// ✅ DO: Pre-allocate collections when size is known
let mut headers = HashMap::with_capacity(header_names.len());
```

### HTTP Method Handling
```rust
// ✅ DO: Handle HTTP error status codes as valid responses
match request.call() {
    Ok(resp) => resp,
    Err(ureq::Error::Status(_code, resp)) => resp,  // 4xx/5xx are valid
    Err(e) => return Err(CurlError::RequestError(e)),
}
```

---

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_get_request

# Run only unit tests (lib.rs)
cargo test --lib

# Run only integration tests
cargo test --test integration_tests
```

### Test Pattern
```rust
// ✅ DO: Use mockito for HTTP mocking
#[test]
fn test_get_request() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_body("Hello, World!")
        .create();

    let client = MinimalCurl::new();
    let response = client.get(&server.url()).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
}
```

---

## Git Workflow

```bash
# Feature branch
git checkout -b feat/new-feature

# Make changes, then test
cargo test
cargo clippy
cargo fmt --check

# Commit
git commit -m "feat: add new feature"

# Bug fix branch
git checkout -b fix/bug-description
```

---

## Boundaries

```bash
# 🚫 NEVER
cargo publish                    # Don't publish without review
git push --force origin main     # DATA LOSS
git add *.exe                    # Binary files
git add target/                  # Build artifacts
rm -rf target/                   # Use cargo clean instead

# ✅ ALWAYS
cargo test                       # Before committing
cargo clippy                     # Check for common mistakes
cargo fmt                        # Format code consistently
cargo build --release            # Verify release build works
```

---

## Protected Paths

| Path | Reason |
|------|--------|
| `target/` | Build artifacts (generated) |
| `Cargo.lock` | Dependency lock (auto-managed) |
| `*.exe` | Binary files |
| `*.pdb` | Debug symbols |

---

## API Reference

### Library Usage
```rust
use bcurl::{MinimalCurl, RequestConfig, HttpMethod};
use std::time::Duration;

// Simple GET
let client = MinimalCurl::new();
let response = client.get("https://httpbin.org/get")?;

// Configured request
let config = RequestConfig::new("https://httpbin.org/post")
    .method(HttpMethod::Post)
    .header("Content-Type", "application/json")
    .data(r#"{"hello": "world"}"#)
    .timeout(Duration::from_secs(10));
let response = client.execute(&config)?;
```

### CLI Usage
```bash
# GET request
bcurl https://example.com

# POST with JSON
bcurl -X POST -d '{"key":"value"}' -H "Content-Type: application/json" https://api.example.com

# Save to file
bcurl -o output.html https://example.com

# HEAD request (headers only)
bcurl -I https://example.com

# Verbose mode
bcurl -v https://example.com
```

---

## Release Profile

The release build uses aggressive optimizations for minimal binary size:

```toml
[profile.release]
lto = "fat"          # Maximum Link-Time Optimization
codegen-units = 1    # Single codegen unit for best optimization
panic = "abort"      # No unwinding - smaller binary
strip = true         # Strip symbols from binary
opt-level = "z"      # Optimize for size
```

---

## Custom Rules
<!-- RULES-SECTION -->
> No custom rules configured. Add rules with `/rules add`.
<!-- END-RULES-SECTION -->

## Common Mistakes
<!-- MISTAKES-SECTION -->
> No mistakes recorded yet. Mistakes will be added here automatically when detected.
<!-- END-MISTAKES-SECTION -->
