use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sha2::{Digest, Sha256, Sha384, Sha512};

// https://www.w3.org/TR/2016/REC-SRI-20160623/#cryptographic-hash-functions
#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy)]
pub enum SRIHashFunction {
  Sha256,
  Sha384,
  Sha512,
}

impl From<String> for SRIHashFunction {
  fn from(s: String) -> Self {
    match s.as_str() {
      "sha256" => Self::Sha256,
      "sha384" => Self::Sha384,
      "sha512" => Self::Sha512,
      _ => panic!("sri hash function only support 'sha256', 'sha384' or 'sha512'"),
    }
  }
}

pub fn compute_integrity(hash_func_names: &Vec<SRIHashFunction>, source: &str) -> String {
  hash_func_names
    .par_iter()
    .map(|hash_func| create_hash(hash_func, source))
    .intersperse(" ".to_string())
    .collect()
}

fn create_hash(hash_func: &SRIHashFunction, source: &str) -> String {
  match hash_func {
    SRIHashFunction::Sha256 => {
      let mut hasher = Sha256::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha256-{}", rspack_base64::encode_to_string(digest))
    }
    SRIHashFunction::Sha384 => {
      let mut hasher = Sha384::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha384-{}", rspack_base64::encode_to_string(digest))
    }
    SRIHashFunction::Sha512 => {
      let mut hasher = Sha512::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha512-{}", rspack_base64::encode_to_string(digest))
    }
  }
}
