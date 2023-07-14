use std::time;

use crate::common::steps;

mod common;

#[test]
fn test_count() {
    let read_timeout = time::Duration::new(1, 250);
    steps(
        vec![
            ("$0 --device lo", None, None),
            ("tcp_listen 127.0.0.1:12345", None, None),
            ("tcp_connect 127.0.0.1:12345", None, None),
            (
                "tcp_connect 127.0.0.1:12345",
                Some(0),
                Some(vec!["127.0.0.1 12345 2"]),
            ),
        ],
        read_timeout,
    );
}
