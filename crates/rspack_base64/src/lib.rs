pub mod base64 {
  use base64_simd::{Base64 as Raw, STANDARD};

  pub struct Base64(Raw);

  impl Base64 {
    pub const fn new() -> Self {
      Self(STANDARD)
    }

    pub fn encode_to_string<D: AsRef<[u8]>>(&self, data: D) -> String {
      self.0.encode_to_string(data)
    }
  }

  impl Default for Base64 {
    fn default() -> Self {
      Self::new()
    }
  }

  pub fn encode_to_string<D: AsRef<[u8]>>(data: D) -> String {
    static BASE64: Base64 = Base64::new();
    BASE64.0.encode_to_string(data)
  }
}

pub use base64::encode_to_string;
