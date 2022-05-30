use thiserror::Error;

#[derive(Error, Debug)]
pub enum RspackCoreError {
  #[error("failed to resolve {0} from {1} due to {2}")]
  ResolveFailed(String, String, String),
}
