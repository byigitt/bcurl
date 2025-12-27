//! Minimal curl - A command-line HTTP client
//!
//! Usage: minimal-curl [OPTIONS] <URL>

use clap::Parser;
use minimal_curl::{parse_header, HttpMethod, MinimalCurl, RequestConfig};
use std::process::ExitCode;
use std::time::Duration;

/// A minimal curl implementation in Rust
#[derive(Parser, Debug)]
#[command(name = "minimal-curl")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to request
    #[arg(required = true)]
    url: String,

    /// HTTP method to use (GET, POST, PUT, DELETE, HEAD, PATCH)
    #[arg(short = 'X', long = "request", default_value = "GET")]
    method: String,

    /// Data to send in the request body
    #[arg(short = 'd', long = "data")]
    data: Option<String>,

    /// Headers to include (can be used multiple times)
    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    /// Follow redirects
    #[arg(short = 'L', long = "location", default_value = "true")]
    follow_redirects: bool,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Write output to file
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Include response headers in output
    #[arg(short = 'i', long = "include")]
    include_headers: bool,

    /// Maximum time in seconds for the request
    #[arg(short = 'm', long = "max-time", default_value = "30")]
    timeout: u64,

    /// Silent mode - don't show progress or errors
    #[arg(short = 's', long = "silent")]
    silent: bool,

    /// Show only the response headers
    #[arg(short = 'I', long = "head")]
    head_only: bool,
}

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

fn main() -> ExitCode {
    let args = Args::parse();

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

    // Build request configuration
    let mut config = RequestConfig::new(&args.url)
        .method(method)
        .follow_redirects(args.follow_redirects)
        .verbose(args.verbose)
        .include_headers(args.include_headers)
        .timeout(Duration::from_secs(args.timeout));

    // Add data if provided
    if let Some(data) = args.data {
        config = config.data(data);
    }

    // Add output file if specified
    if let Some(ref output) = args.output {
        config = config.output_file(output);
    }

    // Parse and add headers
    for header_str in &args.headers {
        match parse_header(header_str) {
            Ok((key, value)) => {
                config = config.header(key, value);
            }
            Err(e) => {
                if !args.silent {
                    eprintln!("Error parsing header: {}", e);
                }
                return ExitCode::FAILURE;
            }
        }
    }

    // Create client and execute request
    let client = MinimalCurl::new();

    match client.execute(&config) {
        Ok(response) => {
            // Print headers if requested
            if args.include_headers || args.head_only {
                println!("HTTP/1.1 {} {}", response.status, response.status_text);
                for (key, value) in &response.headers {
                    println!("{}: {}", key, value);
                }
                println!();
            }

            // Print body (unless head-only or output to file)
            if !args.head_only && args.output.is_none() {
                print!("{}", response.body);
            }

            // Return appropriate exit code
            if response.is_success() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(22) // curl uses 22 for HTTP errors
            }
        }
        Err(e) => {
            if !args.silent {
                eprintln!("Error: {}", e);
            }
            ExitCode::FAILURE
        }
    }
}
