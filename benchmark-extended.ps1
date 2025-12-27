# Extended Benchmark: minimal-curl vs curl
# This script runs comprehensive benchmarks to identify optimization opportunities

$iterations = 5
$minimalCurl = 'C:\Users\user\Desktop\Wired\minimal-curl\target\release\minimal-curl.exe'
$curlExe = 'C:\Windows\System32\curl.exe'

# Results storage
$results = @()

function Measure-Tool {
    param(
        [string]$Name,
        [string]$Tool,
        [string[]]$Args,
        [int]$Iterations = 5
    )

    $times = @()
    for ($i = 0; $i -lt $Iterations; $i++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        & $Tool @Args 2>$null | Out-Null
        $sw.Stop()
        $times += $sw.ElapsedMilliseconds
    }

    return @{
        Avg = [Math]::Round(($times | Measure-Object -Average).Average, 1)
        Min = ($times | Measure-Object -Minimum).Minimum
        Max = ($times | Measure-Object -Maximum).Maximum
    }
}

function Run-Benchmark {
    param(
        [string]$TestName,
        [string[]]$MinimalArgs,
        [string[]]$CurlArgs
    )

    Write-Host "  Testing: $TestName"

    $minimal = Measure-Tool -Name "minimal-curl" -Tool $minimalCurl -Args $MinimalArgs -Iterations $iterations
    $curl = Measure-Tool -Name "curl" -Tool $curlExe -Args $CurlArgs -Iterations $iterations

    $diff = $minimal.Avg - $curl.Avg
    $pct = if ($curl.Avg -ne 0) { [Math]::Round(($diff / $curl.Avg) * 100, 1) } else { 0 }

    return [PSCustomObject]@{
        Test = $TestName
        MinimalAvg = $minimal.Avg
        MinimalMin = $minimal.Min
        MinimalMax = $minimal.Max
        CurlAvg = $curl.Avg
        CurlMin = $curl.Min
        CurlMax = $curl.Max
        Diff = $diff
        DiffPct = $pct
    }
}

Write-Host "=========================================================="
Write-Host "   Extended Benchmark Suite: minimal-curl vs curl"
Write-Host "=========================================================="
Write-Host ""
Write-Host "Iterations per test: $iterations"
Write-Host ""

# Warm up
Write-Host "Warming up..."
& $minimalCurl "https://httpbin.org/get" 2>$null | Out-Null
& $curlExe -s "https://httpbin.org/get" 2>$null | Out-Null
Write-Host ""

# ============================================================
# TEST 1: Basic HTTP Methods
# ============================================================
Write-Host "--- Test Category: HTTP Methods ---"

$results += Run-Benchmark -TestName "GET Request" `
    -MinimalArgs @("https://httpbin.org/get") `
    -CurlArgs @("-s", "https://httpbin.org/get")

$results += Run-Benchmark -TestName "HEAD Request" `
    -MinimalArgs @("-I", "https://httpbin.org/get") `
    -CurlArgs @("-s", "-I", "https://httpbin.org/get")

$results += Run-Benchmark -TestName "POST (small body)" `
    -MinimalArgs @("https://httpbin.org/post", "-X", "POST", "-d", '{"a":1}') `
    -CurlArgs @("-s", "https://httpbin.org/post", "-X", "POST", "-d", '{"a":1}')

$results += Run-Benchmark -TestName "PUT Request" `
    -MinimalArgs @("https://httpbin.org/put", "-X", "PUT", "-d", '{"update":true}') `
    -CurlArgs @("-s", "https://httpbin.org/put", "-X", "PUT", "-d", '{"update":true}')

$results += Run-Benchmark -TestName "DELETE Request" `
    -MinimalArgs @("https://httpbin.org/delete", "-X", "DELETE") `
    -CurlArgs @("-s", "https://httpbin.org/delete", "-X", "DELETE")

Write-Host ""

# ============================================================
# TEST 2: Payload Sizes
# ============================================================
Write-Host "--- Test Category: Payload Sizes ---"

# Small payload
$smallPayload = '{"test": "data"}'
$results += Run-Benchmark -TestName "POST 16 bytes" `
    -MinimalArgs @("https://httpbin.org/post", "-X", "POST", "-d", $smallPayload) `
    -CurlArgs @("-s", "https://httpbin.org/post", "-X", "POST", "-d", $smallPayload)

# Medium payload (1KB)
$mediumPayload = '{"data": "' + ('x' * 1000) + '"}'
$results += Run-Benchmark -TestName "POST 1KB" `
    -MinimalArgs @("https://httpbin.org/post", "-X", "POST", "-d", $mediumPayload) `
    -CurlArgs @("-s", "https://httpbin.org/post", "-X", "POST", "-d", $mediumPayload)

# Larger payload (10KB)
$largePayload = '{"data": "' + ('x' * 10000) + '"}'
$results += Run-Benchmark -TestName "POST 10KB" `
    -MinimalArgs @("https://httpbin.org/post", "-X", "POST", "-d", $largePayload) `
    -CurlArgs @("-s", "https://httpbin.org/post", "-X", "POST", "-d", $largePayload)

Write-Host ""

# ============================================================
# TEST 3: Response Sizes
# ============================================================
Write-Host "--- Test Category: Response Sizes ---"

$results += Run-Benchmark -TestName "Response ~300 bytes" `
    -MinimalArgs @("https://httpbin.org/bytes/300") `
    -CurlArgs @("-s", "https://httpbin.org/bytes/300")

$results += Run-Benchmark -TestName "Response ~1KB" `
    -MinimalArgs @("https://httpbin.org/bytes/1024") `
    -CurlArgs @("-s", "https://httpbin.org/bytes/1024")

$results += Run-Benchmark -TestName "Response ~10KB" `
    -MinimalArgs @("https://httpbin.org/bytes/10240") `
    -CurlArgs @("-s", "https://httpbin.org/bytes/10240")

$results += Run-Benchmark -TestName "Response ~100KB" `
    -MinimalArgs @("https://httpbin.org/bytes/102400") `
    -CurlArgs @("-s", "https://httpbin.org/bytes/102400")

Write-Host ""

# ============================================================
# TEST 4: Headers
# ============================================================
Write-Host "--- Test Category: Headers ---"

$results += Run-Benchmark -TestName "No custom headers" `
    -MinimalArgs @("https://httpbin.org/headers") `
    -CurlArgs @("-s", "https://httpbin.org/headers")

$results += Run-Benchmark -TestName "1 custom header" `
    -MinimalArgs @("https://httpbin.org/headers", "-H", "X-Test: value1") `
    -CurlArgs @("-s", "https://httpbin.org/headers", "-H", "X-Test: value1")

$results += Run-Benchmark -TestName "5 custom headers" `
    -MinimalArgs @("https://httpbin.org/headers", "-H", "X-Test1: v1", "-H", "X-Test2: v2", "-H", "X-Test3: v3", "-H", "X-Test4: v4", "-H", "X-Test5: v5") `
    -CurlArgs @("-s", "https://httpbin.org/headers", "-H", "X-Test1: v1", "-H", "X-Test2: v2", "-H", "X-Test3: v3", "-H", "X-Test4: v4", "-H", "X-Test5: v5")

Write-Host ""

# ============================================================
# TEST 5: Latency Simulation
# ============================================================
Write-Host "--- Test Category: Server Latency ---"

$results += Run-Benchmark -TestName "No delay" `
    -MinimalArgs @("https://httpbin.org/delay/0") `
    -CurlArgs @("-s", "https://httpbin.org/delay/0")

$results += Run-Benchmark -TestName "500ms server delay" `
    -MinimalArgs @("https://httpbin.org/delay/0.5") `
    -CurlArgs @("-s", "https://httpbin.org/delay/0.5")

Write-Host ""

# ============================================================
# TEST 6: Redirects
# ============================================================
Write-Host "--- Test Category: Redirects ---"

$results += Run-Benchmark -TestName "No redirects" `
    -MinimalArgs @("https://httpbin.org/get") `
    -CurlArgs @("-s", "https://httpbin.org/get")

$results += Run-Benchmark -TestName "1 redirect" `
    -MinimalArgs @("-L", "https://httpbin.org/redirect/1") `
    -CurlArgs @("-s", "-L", "https://httpbin.org/redirect/1")

$results += Run-Benchmark -TestName "3 redirects" `
    -MinimalArgs @("-L", "https://httpbin.org/redirect/3") `
    -CurlArgs @("-s", "-L", "https://httpbin.org/redirect/3")

Write-Host ""

# ============================================================
# TEST 7: Cold Start (Process Startup)
# ============================================================
Write-Host "--- Test Category: Cold Start (Startup Time) ---"

# Measure just process startup with fastest possible request
Write-Host "  Testing: Cold start (localhost HEAD)"

# Use a simple endpoint
$coldStartMinimal = @()
$coldStartCurl = @()
for ($i = 0; $i -lt 10; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $minimalCurl "https://httpbin.org/status/200" 2>$null | Out-Null
    $sw.Stop()
    $coldStartMinimal += $sw.ElapsedMilliseconds

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $curlExe -s "https://httpbin.org/status/200" 2>$null | Out-Null
    $sw.Stop()
    $coldStartCurl += $sw.ElapsedMilliseconds
}

$minAvg = [Math]::Round(($coldStartMinimal | Measure-Object -Average).Average, 1)
$curlAvgCS = [Math]::Round(($coldStartCurl | Measure-Object -Average).Average, 1)
$diffCS = $minAvg - $curlAvgCS
$pctCS = if ($curlAvgCS -ne 0) { [Math]::Round(($diffCS / $curlAvgCS) * 100, 1) } else { 0 }

$results += [PSCustomObject]@{
    Test = "Cold start (status check)"
    MinimalAvg = $minAvg
    MinimalMin = ($coldStartMinimal | Measure-Object -Minimum).Minimum
    MinimalMax = ($coldStartMinimal | Measure-Object -Maximum).Maximum
    CurlAvg = $curlAvgCS
    CurlMin = ($coldStartCurl | Measure-Object -Minimum).Minimum
    CurlMax = ($coldStartCurl | Measure-Object -Maximum).Maximum
    Diff = $diffCS
    DiffPct = $pctCS
}

Write-Host ""

# ============================================================
# RESULTS SUMMARY
# ============================================================
Write-Host ""
Write-Host "=========================================================="
Write-Host "                    FULL RESULTS"
Write-Host "=========================================================="
Write-Host ""
Write-Host ("| {0,-28} | {1,10} | {2,10} | {3,10} | {4,8} |" -f "Test", "minimal", "curl", "Diff(ms)", "Diff(%)")
Write-Host ("|" + ("-" * 30) + "|" + ("-" * 12) + "|" + ("-" * 12) + "|" + ("-" * 12) + "|" + ("-" * 10) + "|")

foreach ($r in $results) {
    $diffStr = if ($r.Diff -gt 0) { "+$($r.Diff)" } else { "$($r.Diff)" }
    $pctStr = if ($r.DiffPct -gt 0) { "+$($r.DiffPct)%" } else { "$($r.DiffPct)%" }
    Write-Host ("| {0,-28} | {1,10} | {2,10} | {3,10} | {4,8} |" -f $r.Test, "$($r.MinimalAvg)ms", "$($r.CurlAvg)ms", $diffStr, $pctStr)
}

Write-Host ""
Write-Host ""

# ============================================================
# ANALYSIS: Where minimal-curl is slower
# ============================================================
Write-Host "=========================================================="
Write-Host "              OPTIMIZATION OPPORTUNITIES"
Write-Host "=========================================================="
Write-Host ""

$slower = $results | Where-Object { $_.Diff -gt 20 } | Sort-Object -Property DiffPct -Descending
$faster = $results | Where-Object { $_.Diff -lt -20 } | Sort-Object -Property DiffPct

Write-Host "Areas where minimal-curl is SLOWER (>20ms):"
Write-Host ""
if ($slower.Count -eq 0) {
    Write-Host "  None! minimal-curl is competitive across all tests."
} else {
    foreach ($s in $slower) {
        Write-Host ("  - {0}: +{1}ms ({2}% slower)" -f $s.Test, $s.Diff, $s.DiffPct)
    }
}

Write-Host ""
Write-Host "Areas where minimal-curl is FASTER (>20ms):"
Write-Host ""
if ($faster.Count -eq 0) {
    Write-Host "  None identified."
} else {
    foreach ($f in $faster) {
        Write-Host ("  - {0}: {1}ms ({2}% faster)" -f $f.Test, $f.Diff, [Math]::Abs($f.DiffPct))
    }
}

Write-Host ""
Write-Host ""

# ============================================================
# SPECIFIC RECOMMENDATIONS
# ============================================================
Write-Host "=========================================================="
Write-Host "           POTENTIAL OPTIMIZATIONS TO EXPLORE"
Write-Host "=========================================================="
Write-Host ""

# Calculate averages by category
$getTests = $results | Where-Object { $_.Test -match "GET|HEAD|No redirect" -and $_.Test -notmatch "POST|PUT|DELETE" }
$postTests = $results | Where-Object { $_.Test -match "POST|PUT" }
$responseTests = $results | Where-Object { $_.Test -match "Response" }
$headerTests = $results | Where-Object { $_.Test -match "header" }

$avgGetDiff = if ($getTests) { [Math]::Round(($getTests.Diff | Measure-Object -Average).Average, 1) } else { 0 }
$avgPostDiff = if ($postTests) { [Math]::Round(($postTests.Diff | Measure-Object -Average).Average, 1) } else { 0 }
$avgRespDiff = if ($responseTests) { [Math]::Round(($responseTests.Diff | Measure-Object -Average).Average, 1) } else { 0 }
$avgHeaderDiff = if ($headerTests) { [Math]::Round(($headerTests.Diff | Measure-Object -Average).Average, 1) } else { 0 }

Write-Host "Category Analysis (avg diff vs curl):"
$getSign = if ($avgGetDiff -gt 0) { "+" } else { "" }
$postSign = if ($avgPostDiff -gt 0) { "+" } else { "" }
$respSign = if ($avgRespDiff -gt 0) { "+" } else { "" }
$headerSign = if ($avgHeaderDiff -gt 0) { "+" } else { "" }
Write-Host ("  GET/HEAD requests:  {0}{1}ms" -f $getSign, $avgGetDiff)
Write-Host ("  POST/PUT requests:  {0}{1}ms" -f $postSign, $avgPostDiff)
Write-Host ("  Response handling:  {0}{1}ms" -f $respSign, $avgRespDiff)
Write-Host ("  Header processing:  {0}{1}ms" -f $headerSign, $avgHeaderDiff)
Write-Host ""

Write-Host "Recommendations based on results:"
Write-Host ""

if ($avgGetDiff -gt 30) {
    Write-Host "1. CLIENT INITIALIZATION OVERHEAD"
    Write-Host "   - Consider lazy initialization of the reqwest Client"
    Write-Host "   - Pre-build TLS configuration"
    Write-Host "   - Use rustls instead of native-tls for faster handshakes"
    Write-Host ""
}

if ($avgRespDiff -gt 30) {
    Write-Host "2. RESPONSE BODY HANDLING"
    Write-Host "   - Use streaming instead of buffering entire response"
    Write-Host "   - Avoid unnecessary String allocations (use bytes directly)"
    Write-Host "   - Consider zero-copy parsing where possible"
    Write-Host ""
}

if ($avgHeaderDiff -gt 30) {
    Write-Host "3. HEADER PROCESSING"
    Write-Host "   - Use HeaderMap more efficiently"
    Write-Host "   - Avoid String conversions for header names"
    Write-Host "   - Pre-allocate HashMap capacity"
    Write-Host ""
}

Write-Host "4. GENERAL OPTIMIZATIONS TO CONSIDER:"
Write-Host "   - Enable LTO (Link-Time Optimization) in Cargo.toml"
Write-Host "   - Use 'codegen-units = 1' for better optimization"
Write-Host "   - Consider 'panic = abort' for smaller binary"
Write-Host "   - Profile with 'cargo flamegraph' for hotspots"
Write-Host "   - Use connection pooling for multiple requests"
Write-Host ""

# Binary info
Write-Host ""
Write-Host "=== Binary Comparison ==="
$minimalSize = [Math]::Round((Get-Item $minimalCurl).Length / 1MB, 2)
$curlSize = [Math]::Round((Get-Item $curlExe).Length / 1MB, 2)
Write-Host ("minimal-curl: {0} MB" -f $minimalSize)
Write-Host ("curl:         {0} MB" -f $curlSize)
Write-Host ("Ratio:        {0}x larger" -f [Math]::Round($minimalSize / $curlSize, 1))
Write-Host ""
Write-Host "To reduce binary size:"
Write-Host "  - Strip symbols: 'strip = true' in Cargo.toml"
Write-Host "  - Enable LTO: 'lto = true'"
Write-Host "  - Use 'opt-level = 'z'' for size optimization"
Write-Host "  - Consider using 'cargo-bloat' to find large dependencies"
Write-Host ""
