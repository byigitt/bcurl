//! Integration tests for bcurl

use bcurl::{HttpMethod, MinimalCurl, RequestConfig};
use mockito::{Matcher, Server};
use tempfile::NamedTempFile;

#[test]
fn test_get_request() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("Hello, World!")
        .create();

    let client = MinimalCurl::new();
    let response = client.get(&server.url()).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, "Hello, World!");
    assert!(response.is_success());
}

#[test]
fn test_get_request_with_path() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/api/users")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"users": []}"#)
        .create();

    let client = MinimalCurl::new();
    let url = format!("{}/api/users", server.url());
    let response = client.get(&url).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, r#"{"users": []}"#);
}

#[test]
fn test_post_request_with_data() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/data")
        .match_body(Matcher::Exact(r#"{"name": "test"}"#.to_string()))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id": 1, "name": "test"}"#)
        .create();

    let client = MinimalCurl::new();
    let url = format!("{}/api/data", server.url());
    let response = client.post(&url, Some(r#"{"name": "test"}"#)).unwrap();

    mock.assert();
    assert_eq!(response.status, 201);
    assert!(response.body.contains("\"id\": 1"));
}

#[test]
fn test_put_request() {
    let mut server = Server::new();
    let mock = server
        .mock("PUT", "/api/data/1")
        .match_body(Matcher::Exact(r#"{"name": "updated"}"#.to_string()))
        .with_status(200)
        .with_body(r#"{"id": 1, "name": "updated"}"#)
        .create();

    let client = MinimalCurl::new();
    let url = format!("{}/api/data/1", server.url());
    let response = client.put(&url, Some(r#"{"name": "updated"}"#)).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
}

#[test]
fn test_delete_request() {
    let mut server = Server::new();
    let mock = server
        .mock("DELETE", "/api/data/1")
        .with_status(204)
        .with_body("")
        .create();

    let client = MinimalCurl::new();
    let url = format!("{}/api/data/1", server.url());
    let response = client.delete(&url).unwrap();

    mock.assert();
    assert_eq!(response.status, 204);
}

#[test]
fn test_custom_headers() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/")
        .match_header("Authorization", "Bearer token123")
        .match_header("X-Custom-Header", "custom-value")
        .with_status(200)
        .with_body("Authenticated!")
        .create();

    let client = MinimalCurl::new();
    let config = RequestConfig::new(server.url())
        .header("Authorization", "Bearer token123")
        .header("X-Custom-Header", "custom-value");

    let response = client.execute(&config).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, "Authenticated!");
}

#[test]
fn test_response_headers() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("X-Request-Id", "abc123")
        .with_header("X-Rate-Limit", "100")
        .with_body("OK")
        .create();

    let client = MinimalCurl::new();
    let response = client.get(&server.url()).unwrap();

    mock.assert();
    assert_eq!(
        response.get_header("x-request-id"),
        Some(&"abc123".to_string())
    );
    assert_eq!(
        response.get_header("x-rate-limit"),
        Some(&"100".to_string())
    );
}

#[test]
fn test_404_response() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/not-found")
        .with_status(404)
        .with_body("Not Found")
        .create();

    let client = MinimalCurl::new();
    let url = format!("{}/not-found", server.url());
    let response = client.get(&url).unwrap();

    mock.assert();
    assert_eq!(response.status, 404);
    assert!(!response.is_success());
}

#[test]
fn test_500_response() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/error")
        .with_status(500)
        .with_body("Internal Server Error")
        .create();

    let client = MinimalCurl::new();
    let url = format!("{}/error", server.url());
    let response = client.get(&url).unwrap();

    mock.assert();
    assert_eq!(response.status, 500);
    assert!(!response.is_success());
}

#[test]
fn test_head_request() {
    let mut server = Server::new();
    let mock = server
        .mock("HEAD", "/")
        .with_status(200)
        .with_header("content-length", "1234")
        .create();

    let client = MinimalCurl::new();
    let config = RequestConfig::new(server.url()).method(HttpMethod::Head);

    let response = client.execute(&config).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
    assert!(response.body.is_empty());
}

#[test]
fn test_patch_request() {
    let mut server = Server::new();
    let mock = server
        .mock("PATCH", "/api/data/1")
        .match_body(Matcher::Exact(r#"{"status": "active"}"#.to_string()))
        .with_status(200)
        .with_body(r#"{"id": 1, "status": "active"}"#)
        .create();

    let client = MinimalCurl::new();
    let config = RequestConfig::new(format!("{}/api/data/1", server.url()))
        .method(HttpMethod::Patch)
        .data(r#"{"status": "active"}"#);

    let response = client.execute(&config).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
}

#[test]
fn test_output_to_file() {
    let mut server = Server::new();
    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_body("File content here")
        .create();

    let temp_file = NamedTempFile::new().unwrap();
    let temp_path = temp_file.path().to_str().unwrap().to_string();

    let client = MinimalCurl::new();
    let config = RequestConfig::new(server.url()).output_file(&temp_path);

    let response = client.execute(&config).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);

    let file_content = std::fs::read_to_string(&temp_path).unwrap();
    assert_eq!(file_content, "File content here");
}

#[test]
fn test_large_response() {
    let mut server = Server::new();
    let large_body = "x".repeat(10000);
    let mock = server
        .mock("GET", "/large")
        .with_status(200)
        .with_body(&large_body)
        .create();

    let client = MinimalCurl::new();
    let url = format!("{}/large", server.url());
    let response = client.get(&url).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
    assert_eq!(response.body.len(), 10000);
}

#[test]
fn test_json_content_type() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/api/json")
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"success": true}"#)
        .create();

    let client = MinimalCurl::new();
    let config = RequestConfig::new(format!("{}/api/json", server.url()))
        .method(HttpMethod::Post)
        .header("Content-Type", "application/json")
        .data(r#"{"key": "value"}"#);

    let response = client.execute(&config).unwrap();

    mock.assert();
    assert_eq!(response.status, 200);
    assert_eq!(
        response.get_header("content-type"),
        Some(&"application/json".to_string())
    );
}

#[test]
fn test_multiple_requests() {
    let mut server = Server::new();
    let mock1 = server
        .mock("GET", "/first")
        .with_status(200)
        .with_body("First response")
        .create();
    let mock2 = server
        .mock("GET", "/second")
        .with_status(200)
        .with_body("Second response")
        .create();

    let client = MinimalCurl::new();

    let response1 = client.get(&format!("{}/first", server.url())).unwrap();
    let response2 = client.get(&format!("{}/second", server.url())).unwrap();

    mock1.assert();
    mock2.assert();
    assert_eq!(response1.body, "First response");
    assert_eq!(response2.body, "Second response");
}

#[test]
fn test_request_config_chaining() {
    let mut server = Server::new();
    let mock = server
        .mock("POST", "/chained")
        .match_header("X-Test", "value")
        .match_body(Matcher::Exact("test data".to_string()))
        .with_status(201)
        .with_body("Created")
        .create();

    let client = MinimalCurl::new();
    let config = RequestConfig::new(format!("{}/chained", server.url()))
        .method(HttpMethod::Post)
        .header("X-Test", "value")
        .data("test data")
        .verbose(false)
        .follow_redirects(true);

    let response = client.execute(&config).unwrap();

    mock.assert();
    assert_eq!(response.status, 201);
    assert_eq!(response.body, "Created");
}
