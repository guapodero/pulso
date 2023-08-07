use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IpAddress {
    V6([u8; 16]),
    V4([u8; 4]),
}

impl fmt::Display for IpAddress {
    /// produces a 16 character hex string if "privacy" feature is enabled (default)
    /// otherwise, produces a formatted address
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "privacy")]
        {
            use base16ct::lower::encode_str;
            use blake2::{digest::consts::U8, Blake2b};
            use hmac::{Mac, SimpleHmac};

            type Blake2b64 = Blake2b<U8>;
            type HmacBlake2 = SimpleHmac<Blake2b64>;

            let key = std::env::var("PULSO_SECRET").expect("PULSO_SECRET environment variable");
            let mut hmac = HmacBlake2::new_from_slice(key.as_bytes()).expect("valid key length");
            match *self {
                IpAddress::V6(ref bytes) => hmac.update(bytes),
                IpAddress::V4(ref bytes) => hmac.update(bytes),
            }
            let hash_bytes: [u8; 8] = hmac.finalize().into_bytes().into();
            let mut stack_buf = [0u8; 16];
            f.write_str(encode_str(hash_bytes.as_slice(), &mut stack_buf)?)
        }
        #[cfg(not(feature = "privacy"))]
        {
            use std::net::IpAddr;
            let addr = match *self {
                IpAddress::V6(bytes) => IpAddr::from(bytes),
                IpAddress::V4(bytes) => IpAddr::from(bytes),
            };
            addr.fmt(f)
        }
    }
}
