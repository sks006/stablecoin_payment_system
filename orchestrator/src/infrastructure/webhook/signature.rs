use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn calculate_signature(secret: &str, timestamp: u64, payload: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(format!("{}.{}", timestamp, payload).as_bytes());
    let result = mac.finalize();
    let bytes = result.into_bytes();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
