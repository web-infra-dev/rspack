use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_util::base64;
use sha2::{Digest, Sha256, Sha384, Sha512};

// https://www.w3.org/TR/2016/REC-SRI-20160623/#cryptographic-hash-functions
#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy)]
pub enum SubresourceIntegrityHashFunction {
  Sha256,
  Sha384,
  Sha512,
}

impl TryFrom<String> for SubresourceIntegrityHashFunction {
  type Error = rspack_error::Error;

  fn try_from(value: String) -> Result<Self, rspack_error::Error> {
    match value.as_str() {
      "sha256" => Ok(Self::Sha256),
      "sha384" => Ok(Self::Sha384),
      "sha512" => Ok(Self::Sha512),
      _ => Err(rspack_error::Error::error(format!(
        "Expect SRI hash function to be 'sha256', 'sha384' or 'sha512', but got '{value}'."
      ))),
    }
  }
}

pub(crate) fn compute_integrity(
  hash_func_names: &Vec<SubresourceIntegrityHashFunction>,
  source: &str,
) -> String {
  hash_func_names
    .par_iter()
    .map(|hash_func| create_hash(hash_func, source))
    .intersperse(" ".to_string())
    .collect()
}

fn create_hash(hash_func: &SubresourceIntegrityHashFunction, source: &str) -> String {
  match hash_func {
    SubresourceIntegrityHashFunction::Sha256 => {
      let mut hasher = Sha256::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha256-{}", base64::encode_to_string(digest))
    }
    SubresourceIntegrityHashFunction::Sha384 => {
      let mut hasher = Sha384::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha384-{}", base64::encode_to_string(digest))
    }
    SubresourceIntegrityHashFunction::Sha512 => {
      let mut hasher = Sha512::new();
      hasher.update(source);
      let digest = &hasher.finalize()[..];
      format!("sha512-{}", base64::encode_to_string(digest))
    }
  }
}
