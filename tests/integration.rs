use std::time::Duration;

use crate::common::scenario::Scenario;
use pulso::sensitive::IpAddress;

#[cfg(test)]
mod common;

const LOCALHOST: IpAddress = IpAddress::V4(std::net::Ipv4Addr::LOCALHOST.octets());

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
fn test_connection_limit() {
    let source_id = LOCALHOST.hmac_hex();

    Scenario::default()
        .start("--device lo --connection-limit 2")
        .check_result(None, |o| assert!(o.is_empty()))
        .tcp_listen("127.0.0.1:12345")
        .check_result(None, |o| assert!(o.is_empty()))
        .tcp_connect("127.0.0.1:12345")
        .check_result(None, |o| assert!(o.is_empty()))
        .tcp_connect("127.0.0.1:12345")
        .check_result(Some(0), |o| {
            assert_eq!(o, vec![format!("{source_id} 12345 2")])
        });
}

#[test]
fn test_time_limit() {
    Scenario::default()
        .start("--device lo --time-limit 1")
        .check_result(Some(0), |o| assert!(o.is_empty()))
        .check_duration(|d| assert!(d < Duration::from_millis(1100)));
}
