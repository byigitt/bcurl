# Benchmark: minimal-curl vs curl
$iterations = 10
$testUrl = 'https://httpbin.org/get'
$minimalCurl = 'C:\Users\user\Desktop\Wired\minimal-curl\target\release\minimal-curl.exe'
$curlExe = 'C:\Windows\System32\curl.exe'

Write-Host "=========================================="
Write-Host "   Benchmark: minimal-curl vs curl"
Write-Host "=========================================="
Write-Host ""
Write-Host "Test URL: $testUrl"
Write-Host "Iterations: $iterations"
Write-Host ""

# Warm up both tools
Write-Host "Warming up..."
& $minimalCurl $testUrl 2>$null | Out-Null
& $curlExe -s $testUrl 2>$null | Out-Null
Write-Host ""

# Test 1: Simple GET Request
Write-Host "--- Test 1: Simple GET Request ---"
Write-Host ""

# Benchmark minimal-curl
Write-Host "Testing minimal-curl..."
$minimalTimes = @()
for ($i = 0; $i -lt $iterations; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $minimalCurl $testUrl 2>$null | Out-Null
    $sw.Stop()
    $minimalTimes += $sw.ElapsedMilliseconds
}
$minimalAvg = ($minimalTimes | Measure-Object -Average).Average
$minimalMin = ($minimalTimes | Measure-Object -Minimum).Minimum
$minimalMax = ($minimalTimes | Measure-Object -Maximum).Maximum

# Benchmark curl
Write-Host "Testing curl..."
$curlTimes = @()
for ($i = 0; $i -lt $iterations; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $curlExe -s $testUrl 2>$null | Out-Null
    $sw.Stop()
    $curlTimes += $sw.ElapsedMilliseconds
}
$curlAvg = ($curlTimes | Measure-Object -Average).Average
$curlMin = ($curlTimes | Measure-Object -Minimum).Minimum
$curlMax = ($curlTimes | Measure-Object -Maximum).Maximum

Write-Host ""
Write-Host "=== Results: Simple GET ==="
Write-Host ("minimal-curl: avg={0:F1}ms, min={1}ms, max={2}ms" -f $minimalAvg, $minimalMin, $minimalMax)
Write-Host ("curl:         avg={0:F1}ms, min={1}ms, max={2}ms" -f $curlAvg, $curlMin, $curlMax)
$diff = $minimalAvg - $curlAvg
if ($diff -gt 0) {
    $pct = [Math]::Round(($minimalAvg / $curlAvg - 1) * 100, 1)
    Write-Host ("curl is faster by {0:F1}ms ({1}% slower)" -f [Math]::Abs($diff), $pct)
} else {
    $pct = [Math]::Round(($curlAvg / $minimalAvg - 1) * 100, 1)
    Write-Host ("minimal-curl is faster by {0:F1}ms ({1}% faster)" -f [Math]::Abs($diff), $pct)
}

Write-Host ""
Write-Host ""

# Test 2: GET with headers
$testUrl2 = 'https://httpbin.org/headers'
Write-Host "--- Test 2: GET with Custom Headers ---"
Write-Host ""

# Benchmark minimal-curl with headers
Write-Host "Testing minimal-curl..."
$minimalTimes2 = @()
for ($i = 0; $i -lt $iterations; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $minimalCurl $testUrl2 -H "Accept: application/json" -H "X-Test: benchmark" 2>$null | Out-Null
    $sw.Stop()
    $minimalTimes2 += $sw.ElapsedMilliseconds
}
$minimalAvg2 = ($minimalTimes2 | Measure-Object -Average).Average
$minimalMin2 = ($minimalTimes2 | Measure-Object -Minimum).Minimum
$minimalMax2 = ($minimalTimes2 | Measure-Object -Maximum).Maximum

# Benchmark curl with headers
Write-Host "Testing curl..."
$curlTimes2 = @()
for ($i = 0; $i -lt $iterations; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $curlExe -s $testUrl2 -H "Accept: application/json" -H "X-Test: benchmark" 2>$null | Out-Null
    $sw.Stop()
    $curlTimes2 += $sw.ElapsedMilliseconds
}
$curlAvg2 = ($curlTimes2 | Measure-Object -Average).Average
$curlMin2 = ($curlTimes2 | Measure-Object -Minimum).Minimum
$curlMax2 = ($curlTimes2 | Measure-Object -Maximum).Maximum

Write-Host ""
Write-Host "=== Results: GET with Headers ==="
Write-Host ("minimal-curl: avg={0:F1}ms, min={1}ms, max={2}ms" -f $minimalAvg2, $minimalMin2, $minimalMax2)
Write-Host ("curl:         avg={0:F1}ms, min={1}ms, max={2}ms" -f $curlAvg2, $curlMin2, $curlMax2)
$diff2 = $minimalAvg2 - $curlAvg2
if ($diff2 -gt 0) {
    $pct2 = [Math]::Round(($minimalAvg2 / $curlAvg2 - 1) * 100, 1)
    Write-Host ("curl is faster by {0:F1}ms ({1}% slower)" -f [Math]::Abs($diff2), $pct2)
} else {
    $pct2 = [Math]::Round(($curlAvg2 / $minimalAvg2 - 1) * 100, 1)
    Write-Host ("minimal-curl is faster by {0:F1}ms ({1}% faster)" -f [Math]::Abs($diff2), $pct2)
}

Write-Host ""
Write-Host ""

# Test 3: POST Request
$testUrl3 = 'https://httpbin.org/post'
Write-Host "--- Test 3: POST Request ---"
Write-Host ""

# Benchmark minimal-curl POST
Write-Host "Testing minimal-curl..."
$minimalTimes3 = @()
for ($i = 0; $i -lt $iterations; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $minimalCurl $testUrl3 -X POST -d '{"test": "data"}' -H "Content-Type: application/json" 2>$null | Out-Null
    $sw.Stop()
    $minimalTimes3 += $sw.ElapsedMilliseconds
}
$minimalAvg3 = ($minimalTimes3 | Measure-Object -Average).Average
$minimalMin3 = ($minimalTimes3 | Measure-Object -Minimum).Minimum
$minimalMax3 = ($minimalTimes3 | Measure-Object -Maximum).Maximum

# Benchmark curl POST
Write-Host "Testing curl..."
$curlTimes3 = @()
for ($i = 0; $i -lt $iterations; $i++) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $curlExe -s $testUrl3 -X POST -d '{"test": "data"}' -H "Content-Type: application/json" 2>$null | Out-Null
    $sw.Stop()
    $curlTimes3 += $sw.ElapsedMilliseconds
}
$curlAvg3 = ($curlTimes3 | Measure-Object -Average).Average
$curlMin3 = ($curlTimes3 | Measure-Object -Minimum).Minimum
$curlMax3 = ($curlTimes3 | Measure-Object -Maximum).Maximum

Write-Host ""
Write-Host "=== Results: POST Request ==="
Write-Host ("minimal-curl: avg={0:F1}ms, min={1}ms, max={2}ms" -f $minimalAvg3, $minimalMin3, $minimalMax3)
Write-Host ("curl:         avg={0:F1}ms, min={1}ms, max={2}ms" -f $curlAvg3, $curlMin3, $curlMax3)
$diff3 = $minimalAvg3 - $curlAvg3
if ($diff3 -gt 0) {
    $pct3 = [Math]::Round(($minimalAvg3 / $curlAvg3 - 1) * 100, 1)
    Write-Host ("curl is faster by {0:F1}ms ({1}% slower)" -f [Math]::Abs($diff3), $pct3)
} else {
    $pct3 = [Math]::Round(($curlAvg3 / $minimalAvg3 - 1) * 100, 1)
    Write-Host ("minimal-curl is faster by {0:F1}ms ({1}% faster)" -f [Math]::Abs($diff3), $pct3)
}

Write-Host ""
Write-Host ""

# Summary
Write-Host "=========================================="
Write-Host "              SUMMARY"
Write-Host "=========================================="
Write-Host ""
Write-Host "All times are in milliseconds (lower is better)"
Write-Host ""
Write-Host "| Test               | minimal-curl |     curl | Difference |"
Write-Host "|--------------------|--------------:|---------:|------------|"

$diffStr1 = if (($minimalAvg - $curlAvg) -gt 0) { "+{0:F1}" -f ($minimalAvg - $curlAvg) } else { "{0:F1}" -f ($minimalAvg - $curlAvg) }
$diffStr2 = if (($minimalAvg2 - $curlAvg2) -gt 0) { "+{0:F1}" -f ($minimalAvg2 - $curlAvg2) } else { "{0:F1}" -f ($minimalAvg2 - $curlAvg2) }
$diffStr3 = if (($minimalAvg3 - $curlAvg3) -gt 0) { "+{0:F1}" -f ($minimalAvg3 - $curlAvg3) } else { "{0:F1}" -f ($minimalAvg3 - $curlAvg3) }

Write-Host ("| Simple GET         |     {0,8:F1} | {1,8:F1} | {2,10} |" -f $minimalAvg, $curlAvg, $diffStr1)
Write-Host ("| GET with Headers   |     {0,8:F1} | {1,8:F1} | {2,10} |" -f $minimalAvg2, $curlAvg2, $diffStr2)
Write-Host ("| POST Request       |     {0,8:F1} | {1,8:F1} | {2,10} |" -f $minimalAvg3, $curlAvg3, $diffStr3)
Write-Host ""

# Binary size comparison
$minimalSize = (Get-Item $minimalCurl).Length / 1MB
$curlSize = (Get-Item $curlExe).Length / 1MB

Write-Host ""
Write-Host "=== Binary Size Comparison ==="
Write-Host ("minimal-curl: {0:F2} MB" -f $minimalSize)
Write-Host ("curl:         {0:F2} MB" -f $curlSize)
Write-Host ""

# Version info
Write-Host ""
Write-Host "=== Version Info ==="
& $curlExe --version | Select-Object -First 1
Write-Host "minimal-curl: 0.1.0 (Rust + reqwest)"
Write-Host ""
