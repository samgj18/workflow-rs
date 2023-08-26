use siphasher::sip::SipHasher13;
use std::hash::{Hash, Hasher as _};

#[derive(Debug, Default)]
pub struct Hasher {}

impl Hasher {
    pub fn new() -> Self {
        Hasher {}
    }

    pub fn hash(&self, data: &str) -> u64 {
        // Fast hashing algorithm and secure enough for our use case
        let mut hasher = SipHasher13::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}
