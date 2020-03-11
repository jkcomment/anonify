pub mod aead;
pub mod dh;
pub mod ecies;
pub mod hkdf;
pub mod secrets;

pub const SHA256_OUTPUT_LEN: usize = 256 / 8;

pub trait CryptoRng: rand::RngCore + rand::CryptoRng {}
impl<T> CryptoRng for T
    where T: rand::RngCore + rand::CryptoRng {}
