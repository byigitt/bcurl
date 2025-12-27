//! bcurl - A blazingly fast, minimal HTTP client
//!
//! Usage: bcurl [OPTIONS] <URL>

use bcurl::{parse_header, HttpMethod, MinimalCurl, RequestConfig};
use std::env;
use std::process::ExitCode;
use std::time::Duration;

const VERSION: &str = "0.2.0";
const HELP: &str = r#"bcurl - A blazingly fast, minimal curl alternative in Rust

USAGE:
    bcurl [OPTIONS] <URL>

ARGUMENTS:
    <URL>    URL to request

OPTIONS:
    -X, --request <METHOD>   HTTP method (GET, POST, PUT, DELETE, HEAD, PATCH) [default: GET]
    -d, --data <DATA>        Data to send in request body
    -H, --header <HEADER>    Add header (format: "Name: Value"), can be repeated
    -o, --output <FILE>      Write output to file
    -i, --include            Include response headers in output
    -I, --head               Show only response headers (HEAD request)
    -L, --location           Follow redirects [default: true]
    -m, --max-time <SECS>    Maximum time for request [default: 30]
    -s, --silent             Silent mode
    -v, --verbose            Verbose output
    -h, --help               Show this help
    -V, --version            Show version

EXAMPLES:
    bcurl https://httpbin.org/get
    bcurl -X POST -d '{"key":"value"}' -H "Content-Type: application/json" https://httpbin.org/post
    bcurl -o output.html https://example.com
"#;

struct Args {
    url: String,
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
}

impl Default for Args {
    fn default() -> Self {
        Self {
            url: String::new(),
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
            "-i" | "--include" => result.include_headers = true,
            "-I" | "--head" => result.head_only = true,
            "-L" | "--location" => result.follow_redirects = true,
            "-s" | "--silent" => result.silent = true,
            "-v" | "--verbose" => result.verbose = true,
            arg if arg.starts_with('-') => {
                return Err(format!("Unknown option: {}", arg));
            }
            _ => {
                if result.url.is_empty() {
                    result.url = arg.clone();
                } else {
                    return Err(format!("Unexpected argument: {}", arg));
                }
            }
        }
        i += 1;
    }

    if result.url.is_empty() {
        return Err("URL is required".to_string());
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

    // Create client with appropriate settings
    let client = MinimalCurl::with_config(
        args.follow_redirects,
        Duration::from_secs(args.timeout),
    );

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
