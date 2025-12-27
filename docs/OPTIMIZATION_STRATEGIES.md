# bcurl Optimization Strategies

This document outlines concrete strategies to make bcurl faster than curl.

## Strategy 1: Compression Support (Accept-Encoding)

### The Opportunity
Most web servers support gzip/brotli compression. By requesting compressed responses:
- 10KB HTML → ~2KB gzipped (5× less data)
- Network transfer time reduced proportionally

### Implementation
```rust
// In request building, add:
request = request.set("Accept-Encoding", "gzip, deflate, br");

// In response handling, decompress:
use flate2::read::GzDecoder;
use brotli::Decompressor;

let encoding = response.header("content-encoding");
let body = match encoding {
    Some("gzip") => {
        let mut decoder = GzDecoder::new(reader);
        let mut body = String::new();
        decoder.read_to_string(&mut body)?;
        body
    }
    Some("br") => {
        let mut decoder = Decompressor::new(reader, 4096);
        let mut body = String::new();
        decoder.read_to_string(&mut body)?;
        body
    }
    _ => response.into_string()?,
};
```

### Dependencies
```toml
flate2 = "1.0"  # gzip/deflate (~50KB)
# brotli adds ~200KB, consider making optional
```

### Expected Impact
- **Large responses**: 2-5× faster transfer
- **Small responses**: Minimal impact (compression overhead)
- **Binary size**: +50-200KB

---

## Strategy 2: Connection Pooling (Already Supported!)

### Current State
ureq's `Agent` already implements connection pooling! Multiple requests to the same host reuse connections.

```rust
// This is already in bcurl:
let agent = AgentBuilder::new()
    .tls_connector(Arc::new(tls))
    .build();

// Multiple requests reuse the connection:
agent.get("https://example.com/1").call();  // New connection
agent.get("https://example.com/2").call();  // Reuses connection!
```

### CLI Enhancement
To expose this, add multi-URL support:
```bash
bcurl https://example.com/page1 https://example.com/page2 https://example.com/page3
```

### Expected Impact
- **2nd+ requests**: 50-80% faster (skip DNS, TCP, TLS)
- **Same host**: Major benefit
- **Different hosts**: No benefit

---

## Strategy 3: Parallel Requests

### The Opportunity
curl processes URLs sequentially. bcurl can fetch multiple URLs simultaneously.

### Implementation with Threads
```rust
use std::thread;

fn parallel_fetch(urls: Vec<String>, client: &MinimalCurl) -> Vec<Result<CurlResponse, CurlError>> {
    let client = Arc::new(client.clone());

    let handles: Vec<_> = urls.into_iter().map(|url| {
        let client = Arc::clone(&client);
        thread::spawn(move || {
            let config = RequestConfig::new(&url);
            client.execute(&config)
        })
    }).collect();

    handles.into_iter().map(|h| h.join().unwrap()).collect()
}
```

### CLI Usage
```bash
# Sequential (current)
bcurl url1 url2 url3

# Parallel (new flag)
bcurl --parallel url1 url2 url3
bcurl -P url1 url2 url3

# Batch from file
bcurl --parallel --batch urls.txt
```

### Expected Impact
- **3 URLs**: 3× faster (parallel vs sequential)
- **10 URLs**: Up to 10× faster
- **Network limited**: Eventually saturates bandwidth

---

## Strategy 4: Batch Mode with Connection Reuse

### The Opportunity
Process a list of URLs efficiently with:
1. Single process startup
2. Connection pooling per host
3. Optional parallel processing

### Implementation
```rust
fn batch_mode(url_file: &str, parallel: bool) -> Result<(), CurlError> {
    let urls: Vec<String> = std::fs::read_to_string(url_file)?
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect();

    let client = MinimalCurl::new();

    if parallel {
        // Use thread pool for parallel execution
        let pool = ThreadPool::new(num_cpus::get());
        for url in urls {
            pool.execute(move || {
                let _ = client.get(&url);
            });
        }
        pool.join();
    } else {
        // Sequential with connection reuse
        for url in urls {
            let _ = client.get(&url);
        }
    }
    Ok(())
}
```

### CLI Usage
```bash
# Process URL list
bcurl --batch urls.txt

# Parallel batch
bcurl --batch --parallel urls.txt

# Limit concurrency
bcurl --batch --parallel --max-concurrent 10 urls.txt
```

### Expected Impact
- **100 URLs to same host**: 10-20× faster than curl
- **100 URLs to different hosts**: 5-10× faster (parallel)

---

## Strategy 5: TCP Optimizations

### TCP_NODELAY
Disables Nagle's algorithm, reducing latency for small packets.

```rust
// ureq doesn't expose this directly, but we can configure via native-tls
// For more control, consider using socket2 crate
```

### TCP Buffer Sizing
Larger buffers improve throughput for large transfers.

### Expected Impact
- **Small requests**: 5-10ms improvement
- **Large transfers**: Better throughput

---

## Strategy 6: Faster Memory Allocator

### The Opportunity
Rust's default allocator is good, but specialized allocators can be faster.

### Implementation
```toml
# Cargo.toml
[dependencies]
mimalloc = { version = "0.1", default-features = false }
```

```rust
// main.rs
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

### Expected Impact
- **Startup**: 1-2ms faster
- **Large responses**: Better allocation performance
- **Binary size**: +50KB

---

## Strategy 7: Streaming Output

### The Opportunity
Don't buffer the entire response before outputting.

### Implementation
```rust
pub fn execute_streaming<W: Write>(
    &self,
    config: &RequestConfig,
    output: &mut W
) -> Result<CurlResponse, CurlError> {
    let response = /* make request */;

    // Stream directly to output
    let mut reader = response.into_reader();
    std::io::copy(&mut reader, output)?;

    Ok(/* response metadata */)
}
```

### Expected Impact
- **Large responses**: First byte to output faster
- **Memory usage**: Constant instead of O(response_size)

---

## Implementation Roadmap

### Phase 1: Quick Wins (Low effort, High impact)
1. [ ] Add compression support (gzip)
2. [ ] Enable TCP_NODELAY if possible
3. [ ] Add multi-URL support in CLI

### Phase 2: Parallel Processing (Medium effort, Very high impact)
4. [ ] Add --parallel flag for concurrent requests
5. [ ] Add --batch mode for URL lists
6. [ ] Add --max-concurrent limit

### Phase 3: Polish (Medium effort, Medium impact)
7. [ ] Streaming output for large responses
8. [ ] Optional mimalloc allocator
9. [ ] Progress indicator for batch mode

### Phase 4: Advanced (High effort, Situational impact)
10. [ ] HTTP/2 support (via different backend)
11. [ ] DNS caching
12. [ ] TLS session resumption

## Benchmarking Each Optimization

For each optimization, measure:
1. **Single request latency**: Should not regress
2. **Multi-request throughput**: Target 2-10× improvement
3. **Binary size**: Track growth
4. **Memory usage**: Peak and average

```powershell
# Benchmark script
Measure-Command { bcurl https://example.com }
Measure-Command { bcurl --parallel url1 url2 url3 }
Measure-Command { bcurl --batch urls.txt }
```

See [BEATING_CURL.md](./BEATING_CURL.md) for the summary of how to beat curl.
