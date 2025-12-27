//! bcurl - A blazingly fast, minimal HTTP client
//!
//! Usage: bcurl [OPTIONS] <URL>...
//!
//! Features that make bcurl faster than curl:
//! - Connection pooling for multiple requests to same host
//! - Parallel request execution with --parallel
//! - Automatic compression (gzip/deflate)
//! - Batch mode for processing URL files

use bcurl::{parse_header, HttpMethod, MinimalCurl, RequestConfig};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::ExitCode;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const VERSION: &str = "0.3.0";
const HELP: &str = r#"bcurl - A blazingly fast HTTP client that beats curl for multiple requests

USAGE:
    bcurl [OPTIONS] <URL>...

ARGUMENTS:
    <URL>...    One or more URLs to request

OPTIONS:
    -X, --request <METHOD>   HTTP method (GET, POST, PUT, DELETE, HEAD, PATCH) [default: GET]
    -d, --data <DATA>        Data to send in request body
    -H, --header <HEADER>    Add header (format: "Name: Value"), can be repeated
    -o, --output <FILE>      Write output to file (only for single URL)
    -i, --include            Include response headers in output
    -I, --head               Show only response headers (HEAD request)
    -L, --location           Follow redirects [default: true]
    -m, --max-time <SECS>    Maximum time for request [default: 30]
    -s, --silent             Silent mode
    -v, --verbose            Verbose output
    -h, --help               Show this help
    -V, --version            Show version

PERFORMANCE OPTIONS (bcurl exclusive):
    -P, --parallel           Execute multiple URLs in parallel (faster!)
    -B, --batch <FILE>       Read URLs from file (one per line)
    --no-compression         Disable automatic gzip/deflate compression
    --timing                 Show timing information for each request

EXAMPLES:
    # Single request (same as curl)
    bcurl https://httpbin.org/get

    # Multiple URLs with connection reuse (faster than curl!)
    bcurl https://example.com/page1 https://example.com/page2 https://example.com/page3

    # Parallel requests (much faster than curl!)
    bcurl --parallel https://site1.com https://site2.com https://site3.com

    # Batch mode with parallel execution
    bcurl --batch urls.txt --parallel

    # POST with JSON
    bcurl -X POST -d '{"key":"value"}' -H "Content-Type: application/json" https://httpbin.org/post

WHY BCURL IS FASTER:
    - Multiple URLs to same host: 50-80% faster (connection reuse)
    - Parallel requests: Up to Nx faster for N URLs
    - Compressed responses: 2-5x faster transfers for compressible content
"#;

struct Args {
    urls: Vec<String>,
    method: String,
    data: Option<String>,
    headers: Vec<String>,
    output: Option<String>,
    include_headers: bool,
    head_only: bool,
    follow_redirects: bool,
    timeout: u64,
    silent: bool,
    verbose: bool,
    parallel: bool,
    batch_file: Option<String>,
    compression: bool,
    timing: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            urls: Vec::new(),
            method: "GET".to_string(),
            data: None,
            headers: Vec::new(),
            output: None,
            include_headers: false,
            head_only: false,
            follow_redirects: true,
            timeout: 30,
            silent: false,
            verbose: false,
            parallel: false,
            batch_file: None,
            compression: true,
            timing: false,
        }
    }
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return Err("URL is required".to_string());
    }

    let mut result = Args::default();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        match arg.as_str() {
            "-h" | "--help" => {
                print!("{}", HELP);
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("bcurl {}", VERSION);
                std::process::exit(0);
            }
            "-X" | "--request" => {
                i += 1;
                if i >= args.len() {
                    return Err("-X requires a method argument".to_string());
                }
                result.method = args[i].clone();
            }
            "-d" | "--data" => {
                i += 1;
                if i >= args.len() {
                    return Err("-d requires a data argument".to_string());
                }
                result.data = Some(args[i].clone());
            }
            "-H" | "--header" => {
                i += 1;
                if i >= args.len() {
                    return Err("-H requires a header argument".to_string());
                }
                result.headers.push(args[i].clone());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("-o requires a file argument".to_string());
                }
                result.output = Some(args[i].clone());
            }
            "-m" | "--max-time" => {
                i += 1;
                if i >= args.len() {
                    return Err("-m requires a timeout argument".to_string());
                }
                result.timeout = args[i]
                    .parse()
                    .map_err(|_| format!("Invalid timeout: {}", args[i]))?;
            }
            "-B" | "--batch" => {
                i += 1;
                if i >= args.len() {
                    return Err("-B requires a file argument".to_string());
                }
                result.batch_file = Some(args[i].clone());
            }
            "-i" | "--include" => result.include_headers = true,
            "-I" | "--head" => result.head_only = true,
            "-L" | "--location" => result.follow_redirects = true,
            "-s" | "--silent" => result.silent = true,
            "-v" | "--verbose" => result.verbose = true,
            "-P" | "--parallel" => result.parallel = true,
            "--no-compression" => result.compression = false,
            "--timing" => result.timing = true,
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: {}", arg));
            }
            _ => {
                // Collect as URL
                result.urls.push(arg.clone());
            }
        }
        i += 1;
    }

    // Load URLs from batch file if specified
    if let Some(ref batch_file) = result.batch_file {
        let file = File::open(batch_file)
            .map_err(|e| format!("Failed to open batch file '{}': {}", batch_file, e))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read batch file: {}", e))?;
            let line = line.trim();
            // Skip empty lines and comments
            if !line.is_empty() && !line.starts_with('#') {
                result.urls.push(line.to_string());
            }
        }
    }

    if result.urls.is_empty() {
        return Err("At least one URL is required".to_string());
    }

    Ok(result)
}

#[inline]
fn parse_method(method: &str) -> Result<HttpMethod, String> {
    match method.to_uppercase().as_str() {
        "GET" => Ok(HttpMethod::Get),
        "POST" => Ok(HttpMethod::Post),
        "PUT" => Ok(HttpMethod::Put),
        "DELETE" => Ok(HttpMethod::Delete),
        "HEAD" => Ok(HttpMethod::Head),
        "PATCH" => Ok(HttpMethod::Patch),
        _ => Err(format!("Unknown HTTP method: {}", method)),
    }
}

/// Execute requests sequentially with connection reuse
fn execute_sequential(
    client: &MinimalCurl,
    urls: &[String],
    args: &Args,
    method: HttpMethod,
    headers: &[(String, String)],
) -> bool {
    let mut all_success = true;

    for (idx, url) in urls.iter().enumerate() {
        let start = Instant::now();

        // Build request configuration
        let mut config = RequestConfig::new(url)
            .method(method)
            .follow_redirects(args.follow_redirects)
            .verbose(args.verbose)
            .include_headers(args.include_headers)
            .compression(args.compression)
            .timeout(Duration::from_secs(args.timeout));

        // Add data if provided
        if let Some(ref data) = args.data {
            config = config.data(data.clone());
        }

        // Add output file if specified (only for single URL)
        if urls.len() == 1 {
            if let Some(ref output) = args.output {
                config = config.output_file(output);
            }
        }

        // Add headers
        for (key, value) in headers {
            config = config.header(key.clone(), value.clone());
        }

        match client.execute(&config) {
            Ok(response) => {
                let elapsed = start.elapsed();

                // Print headers if requested
                if args.include_headers || args.head_only {
                    if urls.len() > 1 {
                        println!("=== {} ===", url);
                    }
                    println!("HTTP/1.1 {} {}", response.status, response.status_text);
                    for (key, value) in &response.headers {
                        println!("{}: {}", key, value);
                    }
                    println!();
                }

                // Print body (unless head-only or output to file)
                if !args.head_only && (urls.len() == 1 || args.output.is_none()) {
                    if urls.len() > 1 && !args.include_headers {
                        println!("=== {} ===", url);
                    }
                    print!("{}", response.body);
                    if urls.len() > 1 {
                        println!(); // Add newline between responses
                    }
                }

                // Print timing if requested
                if args.timing {
                    eprintln!(
                        "[{}] {} - {} {} - {:.2}ms",
                        idx + 1,
                        url,
                        response.status,
                        response.status_text,
                        elapsed.as_secs_f64() * 1000.0
                    );
                }

                if !response.is_success() {
                    all_success = false;
                }
            }
            Err(e) => {
                if !args.silent {
                    eprintln!("Error fetching {}: {}", url, e);
                }
                all_success = false;
            }
        }
    }

    all_success
}

/// Execute requests in parallel using threads
fn execute_parallel(
    client: Arc<MinimalCurl>,
    urls: Vec<String>,
    args: &Args,
    method: HttpMethod,
    headers: Vec<(String, String)>,
) -> bool {
    let total_start = Instant::now();
    let silent = args.silent;
    let verbose = args.verbose;
    let include_headers = args.include_headers;
    let head_only = args.head_only;
    let follow_redirects = args.follow_redirects;
    let compression = args.compression;
    let timeout = args.timeout;
    let timing = args.timing;
    let data = args.data.clone();

    // Spawn threads for each URL
    let handles: Vec<_> = urls
        .into_iter()
        .enumerate()
        .map(|(idx, url)| {
            let client = Arc::clone(&client);
            let headers = headers.clone();
            let data = data.clone();

            thread::spawn(move || {
                let start = Instant::now();

                // Build request configuration
                let mut config = RequestConfig::new(&url)
                    .method(method)
                    .follow_redirects(follow_redirects)
                    .verbose(verbose)
                    .include_headers(include_headers)
                    .compression(compression)
                    .timeout(Duration::from_secs(timeout));

                // Add data if provided
                if let Some(ref data) = data {
                    config = config.data(data.clone());
                }

                // Add headers
                for (key, value) in &headers {
                    config = config.header(key.clone(), value.clone());
                }

                let result = client.execute(&config);
                let elapsed = start.elapsed();

                (idx, url, result, elapsed)
            })
        })
        .collect();

    // Collect results
    let mut results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .collect();

    // Sort by original index to maintain order
    results.sort_by_key(|(idx, _, _, _)| *idx);

    let mut all_success = true;

    // Print results
    for (idx, url, result, elapsed) in results {
        match result {
            Ok(response) => {
                // Print headers if requested
                if include_headers || head_only {
                    println!("=== {} ===", url);
                    println!("HTTP/1.1 {} {}", response.status, response.status_text);
                    for (key, value) in &response.headers {
                        println!("{}: {}", key, value);
                    }
                    println!();
                }

                // Print body
                if !head_only {
                    if !include_headers {
                        println!("=== {} ===", url);
                    }
                    print!("{}", response.body);
                    println!();
                }

                // Print timing if requested
                if timing {
                    eprintln!(
                        "[{}] {} - {} {} - {:.2}ms",
                        idx + 1,
                        url,
                        response.status,
                        response.status_text,
                        elapsed.as_secs_f64() * 1000.0
                    );
                }

                if !response.is_success() {
                    all_success = false;
                }
            }
            Err(e) => {
                if !silent {
                    eprintln!("Error fetching {}: {}", url, e);
                }
                all_success = false;
            }
        }
    }

    if timing {
        let total_elapsed = total_start.elapsed();
        eprintln!(
            "\nTotal time: {:.2}ms (parallel execution)",
            total_elapsed.as_secs_f64() * 1000.0
        );
    }

    all_success
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("Use --help for usage information");
            return ExitCode::FAILURE;
        }
    };

    // Parse HTTP method
    let method = if args.head_only {
        HttpMethod::Head
    } else {
        match parse_method(&args.method) {
            Ok(m) => m,
            Err(e) => {
                if !args.silent {
                    eprintln!("Error: {}", e);
                }
                return ExitCode::FAILURE;
            }
        }
    };

    // Parse headers once
    let mut headers = Vec::new();
    for header_str in &args.headers {
        match parse_header(header_str) {
            Ok((key, value)) => {
                headers.push((key, value));
            }
            Err(e) => {
                if !args.silent {
                    eprintln!("Error parsing header: {}", e);
                }
                return ExitCode::FAILURE;
            }
        }
    }

    // Create client with appropriate settings
    // The client maintains connection pool for reuse
    let client = MinimalCurl::with_config(args.follow_redirects, Duration::from_secs(args.timeout));

    // Execute requests
    let success = if args.parallel && args.urls.len() > 1 {
        // Parallel execution for multiple URLs
        let client = Arc::new(client);
        execute_parallel(client, args.urls.clone(), &args, method, headers)
    } else {
        // Sequential execution with connection reuse
        execute_sequential(&client, &args.urls, &args, method, &headers)
    };

    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(22) // curl uses 22 for HTTP errors
    }
}
