# How bcurl Can Beat curl

## Executive Summary

**Can bcurl be faster than curl?**

| Scenario | Can Beat curl? | How |
|----------|----------------|-----|
| Single request to remote server | **No** | Network-bound; both tools equal |
| Multiple requests to same host | **Yes** | Connection pooling saves TCP+TLS overhead |
| Parallel requests | **Yes** | Thread-based concurrency |
| Large compressible responses | **Yes** | Built-in gzip/brotli decompression |
| Batch processing URLs | **Yes** | Single process + connection reuse |
| localhost/fast network | **Maybe** | Micro-optimizations matter more |

## The Network-Bound Reality

For a typical HTTPS request to a remote server:

```
Total time: ~700ms
├── Network RTT (3-4 round trips): ~600ms (86%)
├── TLS crypto operations: ~50ms (7%)
├── DNS lookup: ~30ms (4%)
└── Local processing: ~20ms (3%)
```

**The 86% we cannot control.** Both curl and bcurl must wait for network round trips.

## Where bcurl WILL Win

### 1. Multiple Requests to Same Host

```
Scenario: Fetch 5 pages from example.com

curl (5 separate invocations):
  Request 1: DNS + TCP + TLS + HTTP = 700ms
  Request 2: DNS + TCP + TLS + HTTP = 700ms
  Request 3: DNS + TCP + TLS + HTTP = 700ms
  Request 4: DNS + TCP + TLS + HTTP = 700ms
  Request 5: DNS + TCP + TLS + HTTP = 700ms
  Total: 3500ms

bcurl (connection reuse):
  Request 1: DNS + TCP + TLS + HTTP = 700ms
  Request 2: HTTP only (reuse) = 200ms
  Request 3: HTTP only (reuse) = 200ms
  Request 4: HTTP only (reuse) = 200ms
  Request 5: HTTP only (reuse) = 200ms
  Total: 1500ms

Winner: bcurl is 2.3× faster!
```

### 2. Parallel Requests

```
Scenario: Fetch 3 independent URLs

curl (sequential):
  URL 1: 700ms
  URL 2: 700ms
  URL 3: 700ms
  Total: 2100ms

bcurl --parallel:
  URL 1: ─────────[700ms]
  URL 2: ─────────[700ms]
  URL 3: ─────────[700ms]
  Total: 700ms (all concurrent)

Winner: bcurl is 3× faster!
```

### 3. Compressed Responses

```
Scenario: Fetch 50KB HTML page

curl (no Accept-Encoding by default):
  Transfer: 50KB
  At 10 Mbps: 40ms transfer time

bcurl (with gzip):
  Transfer: 10KB (compressed)
  At 10 Mbps: 8ms transfer time

Winner: bcurl saves 32ms per request
        For 100 requests: 3.2 seconds saved!
```

### 4. Batch URL Processing

```
Scenario: Fetch 100 URLs from a file

curl (100 process invocations):
  100 × process startup = 500ms overhead
  100 × new connections = slow

bcurl --batch urls.txt:
  1 × process startup = 5ms overhead
  Connection pooling per host
  Optional parallel processing

Winner: bcurl eliminates ~500ms of startup overhead
```

## What We're Implementing

### Phase 1: Compression (Implemented)
- Accept-Encoding: gzip, deflate
- Automatic decompression
- Benefit: Faster transfers for compressible content

### Phase 2: Multi-URL Support (Implemented)
- `bcurl url1 url2 url3`
- Connection reuse between requests
- Benefit: 50-80% faster for 2nd+ requests to same host

### Phase 3: Parallel Mode (Implemented)
- `bcurl --parallel url1 url2 url3`
- Thread-based concurrent requests
- Benefit: N URLs in time of 1 URL

### Phase 4: Batch Mode (Implemented)
- `bcurl --batch urls.txt`
- Combine with --parallel for maximum speed
- Benefit: Process thousands of URLs efficiently

## Benchmark Results (Expected)

| Test Case | curl | bcurl | Improvement |
|-----------|------|-------|-------------|
| 1 GET request | 700ms | 700ms | 0% |
| 5 GETs same host | 3500ms | 1500ms | **57% faster** |
| 5 GETs parallel | 3500ms | 700ms | **80% faster** |
| 100 URLs batch | ~70s | ~15s | **78% faster** |
| 50KB compressed | 740ms | 708ms | **4% faster** |

## The Honest Assessment

### What curl does better:
- More protocols (FTP, SFTP, LDAP, etc.)
- More authentication methods
- More mature error handling
- Ubiquitous availability

### What bcurl does better:
- Smaller binary (504KB vs 553KB)
- Built-in connection reuse
- Native parallel requests
- Automatic compression
- Modern Rust memory safety

### Best use cases for bcurl:
1. **API clients**: Multiple requests to same endpoint
2. **Web scrapers**: Batch URL processing
3. **CI/CD pipelines**: Health checks, deployments
4. **Microservices**: Service-to-service HTTP calls
5. **Embedded systems**: Small footprint matters

## Conclusion

**bcurl cannot beat curl on a single request** - both are network-bound.

**bcurl CAN beat curl significantly when:**
- Making multiple requests
- Using parallel processing
- Leveraging compression
- Processing URL batches

The key insight: **curl is optimized for single requests; bcurl can be optimized for throughput.**

---

See implementation in:
- [PERFORMANCE_ANALYSIS.md](./PERFORMANCE_ANALYSIS.md) - Detailed timing breakdown
- [OPTIMIZATION_STRATEGIES.md](./OPTIMIZATION_STRATEGIES.md) - Implementation guide
