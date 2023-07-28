use base16ct::lower::encode_string;
use blake2::{digest::consts::U8, Blake2b};
use hmac::{Mac, SimpleHmac};

type Blake2b64 = Blake2b<U8>;
type HmacBlake2 = SimpleHmac<Blake2b64>;

#[derive(Debug, Clone)]
pub enum IpAddress {
    V4([u8; 4]),
}

impl IpAddress {
    pub fn hmac_hex(&self) -> String {
        match self {
            &IpAddress::V4(ref octets) => {
                let key = std::env::var("PULSO_SECRET").unwrap();
                let mut hmac =
                    HmacBlake2::new_from_slice(key.as_bytes()).expect("valid key length");
                hmac.update(octets);
                let hash_bytes: [u8; 8] = hmac.finalize().into_bytes().into();
                encode_string(hash_bytes.as_slice())
            }
        }
    }
}
