use std::fmt;

use base16ct::lower::encode_str;
use blake2::{digest::consts::U8, Blake2b};
use hmac::{Mac, SimpleHmac};

type Blake2b64 = Blake2b<U8>;
type HmacBlake2 = SimpleHmac<Blake2b64>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IpAddress {
    V6([u8; 16]),
    V4([u8; 4]),
}

impl fmt::Display for IpAddress {
    /// a string of 16 hexadecimal characters
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = std::env::var("PULSO_SECRET").unwrap();
        let mut hmac = HmacBlake2::new_from_slice(key.as_bytes()).expect("valid key length");
        match self {
            &IpAddress::V6(ref bytes) => hmac.update(bytes),
            &IpAddress::V4(ref bytes) => hmac.update(bytes),
        }
        let hash_bytes: [u8; 8] = hmac.finalize().into_bytes().into();
        let mut stack_buf = [0u8; 16];
        f.write_str(encode_str(hash_bytes.as_slice(), &mut stack_buf)?)
    }
}
