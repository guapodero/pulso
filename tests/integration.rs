use crate::common::scenario::Scenario;

#[cfg(test)]
mod common;

#[test]
fn test_help() {
    Scenario::default()
        .start("--help")
        .check_result(Some(0), |o| assert!(o.contains(&"TCP connection counter")));
}

#[test]
fn test_unexpected_args() {
    Scenario::default()
        .start("")
        .check_result(Some(2), |o| assert!(o.is_empty()));
}

#[test]
fn test_invalid_interface() {
    Scenario::default()
        .env("RUST_BACKTRACE", "0")
        .start("--device kungfu")
        .check_result(Some(1), |o| assert!(o.is_empty()));
}

#[test]
fn test_count() {
    Scenario::default()
        .start("--device lo")
        .check_result(None, |o| assert!(o.is_empty()))
        .tcp_listen("127.0.0.1:12345")
        .check_result(None, |o| assert!(o.is_empty()))
        .tcp_connect("127.0.0.1:12345")
        .check_result(None, |o| assert!(o.is_empty()))
        .tcp_connect("127.0.0.1:12345")
        .check_result(Some(0), |o| assert_eq!(o, vec!["127.0.0.1 12345 2"]));
}
