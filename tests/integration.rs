use std::time::Duration;

use crate::common::scenario::Scenario;
use pulso::sensitive::IpAddress;

#[cfg(test)]
mod common;

const LOCALHOST_V6: IpAddress = IpAddress::V6(1u128.swap_bytes().to_ne_bytes());
const LOCALHOST_V4: IpAddress = IpAddress::V4([127, 0, 0, 1]);

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
fn test_connection_limit_ipv6() {
    let source_id = LOCALHOST_V6.hmac_hex();

    Scenario::default()
        .start("--device lo --connection-limit 1")
        .check_result(None, |o| assert!(o.is_empty()))
        .tcp_listen("[::1]:12345")
        .tcp_connect("[::1]:12345")
        .check_result(Some(0), |o| {
            assert_eq!(o, vec![format!("{source_id}:1 12345:1")])
        });
}

#[test]
fn test_connection_limit_ipv4() {
    let source_id = LOCALHOST_V4.hmac_hex();

    Scenario::default()
        .start("--device lo --connection-limit 1")
        .check_result(None, |o| assert!(o.is_empty()))
        .tcp_listen("127.0.0.1:12345")
        .tcp_connect("127.0.0.1:12345")
        .check_result(Some(0), |o| {
            assert_eq!(o, vec![format!("{source_id}:1 12345:1")])
        });
}

#[test]
fn test_time_limit() {
    Scenario::default()
        .start("--device lo --time-limit 1")
        .check_result(Some(0), |o| assert!(o.is_empty()))
        .check_duration(|d| assert!(d < Duration::from_millis(1100)));
}
