# bcurl Performance Analysis

## Where Does HTTP Request Time Go?

When making a single HTTP request, time is distributed across these phases:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        HTTP REQUEST TIMELINE                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Process    DNS      TCP         TLS            HTTP          Response   │
│  Startup  Lookup  Handshake   Handshake    Request/Wait     Processing  │
│    │        │        │           │              │               │        │
│    ▼        ▼        ▼           ▼              ▼               ▼        │
│  ┌───┐   ┌────┐   ┌────┐     ┌──────┐      ┌───────┐       ┌──────┐    │
│  │5ms│   │20ms│   │ RTT│     │1-2RTT│      │Server │       │Parse │    │
│  └───┘   └────┘   └────┘     └──────┘      │Process│       │Output│    │
│                                             └───────┘       └──────┘    │
│                                                                           │
│  ~5ms     ~20ms    ~30ms      ~60ms        Variable         ~5ms        │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘

Total typical time: 600-900ms (dominated by network RTT)
```

## Breakdown by Component

### 1. Process Startup (~1-5ms)
- Loading executable into memory
- Dynamic library loading (native-tls uses OS crypto)
- Initializing the Rust runtime
- Parsing command-line arguments

**bcurl advantage**: Small 504KB binary loads fast

### 2. DNS Resolution (~10-50ms)
- Converting hostname (e.g., httpbin.org) to IP address
- Typically cached by OS for subsequent requests
- First request to a new domain is slowest

**Optimization opportunity**: DNS caching, async DNS

### 3. TCP Handshake (1 RTT, ~20-50ms)
- SYN → SYN-ACK → ACK
- Establishes connection to server

**Optimization opportunity**: TCP Fast Open (TFO), connection pooling

### 4. TLS Handshake (1-2 RTT, ~40-100ms)
- Certificate exchange and validation
- Key negotiation
- TLS 1.3 reduces this to 1 RTT (or 0-RTT with resumption)

**This is often the BIGGEST controllable factor**

### 5. HTTP Request/Response (1+ RTT)
- Send request headers and body
- Wait for server processing
- Receive response headers and body

**Server processing time is outside our control**

### 6. Response Processing (~1-10ms)
- Reading response body
- String allocation and parsing
- Output to stdout or file

**Optimization opportunity**: Streaming, zero-copy

## Measured Results (httpbin.org)

| Phase | bcurl | curl | Notes |
|-------|-------|------|-------|
| Cold start | ~10ms | ~8ms | Both are fast |
| GET request | ~700ms | ~700ms | Network bound |
| POST request | ~700ms | ~700ms | Network bound |
| HEAD request | ~650ms | ~650ms | Network bound |

**Conclusion**: For single requests, both tools are **network-bound**. The 600ms+ times are dominated by network RTT to httpbin.org (likely 200-300ms RTT for each phase).

## The Hard Truth About Single Requests

For a **single HTTP request** to a remote server:

```
Time = Startup + DNS + TCP_RTT + TLS_RTT×1.5 + Server_Time + Transfer_Time
```

Most of this is **network latency** that neither curl nor bcurl can control.

To beat curl on single requests, we need to optimize the ~10-20ms of local processing, which gives minimal benefit when total time is 700ms.

## Where bcurl CAN Beat curl

### Scenario 1: Multiple Requests to Same Host
```
curl:   3 requests × 700ms = 2100ms (new connection each time)
bcurl:  700ms + 100ms + 100ms = 900ms (connection reuse)
```

### Scenario 2: Compressed Responses
```
10KB response:
  curl (no compression):  Transfer ~10KB
  bcurl (gzip enabled):   Transfer ~2KB → 5× faster transfer
```

### Scenario 3: Parallel Requests
```
curl:   3 sequential requests = 2100ms
bcurl:  3 parallel requests = 700ms (3× faster!)
```

### Scenario 4: Batch Mode
```
curl:   100 URLs = 100 process startups
bcurl:  100 URLs = 1 process startup + connection reuse
```

## Optimization Priority Matrix

| Optimization | Single Request | Multiple Requests | Implementation Effort |
|--------------|----------------|-------------------|----------------------|
| Compression (gzip/br) | Medium | High | Low |
| Connection pooling | None | **Very High** | Low (ureq built-in) |
| Parallel requests | None | **Very High** | Medium |
| Batch mode | None | **Very High** | Medium |
| TCP_NODELAY | Low | Medium | Low |
| Faster allocator | Low | Medium | Low |
| Streaming output | Low | Medium | Medium |

## Recommendations

1. **For single requests**: Accept that we're network-bound; focus on not being slower
2. **For multiple requests**: Implement connection pooling (ureq already does this with Agent)
3. **For high throughput**: Add parallel request support
4. **For bandwidth-limited**: Enable compression by default
5. **For batch processing**: Add batch mode with URL list input

See [OPTIMIZATION_STRATEGIES.md](./OPTIMIZATION_STRATEGIES.md) for implementation details.
