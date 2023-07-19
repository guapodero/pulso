use crate::common::steps;
use crate::common::Output::{F, V, X};

#[cfg(test)]
mod common;

#[test]
fn test_help() {
    steps(vec![(
        "$0 --help",
        V(0),
        F(|lines| assert!(lines.contains(&"TCP connection counter".to_string()))),
    )]);
}

#[test]
fn test_missing_args() {
    steps(vec![("$0", V(2), X)]);
}

#[test]
fn test_invalid_interface() {
    steps(vec![
        ("env RUST_BACKTRACE=0", X, X),
        ("$0 --device kungfu", V(1), X),
    ]);
}

#[test]
fn test_count() {
    steps(vec![
        ("$0 --device lo", X, X),
        ("tcp_listen 127.0.0.1:12345", X, X),
        ("tcp_connect 127.0.0.1:12345", X, X),
        (
            "tcp_connect 127.0.0.1:12345",
            V(0),
            V(vec!["127.0.0.1 12345 2".to_string()]),
        ),
    ]);
}
