use regress::Regex;
use rspack_error::{Error, error};

#[derive(Debug, Clone)]
pub struct RspackRegressRegex(pub Regex);

impl RspackRegressRegex {
  pub fn test(&self, text: &str) -> bool {
    self.0.find(text).is_some()
  }
  pub fn with_flags(source: &str, flags: &str) -> Result<Self, Error> {
    let mut chars = flags.chars().collect::<Vec<char>>();
    chars.sort_unstable();
    let regex = match Regex::with_flags(source, flags) {
      Ok(regex) => Ok(regex),
      Err(err) => Err(error!(
        "Can't construct regex `/{source}/{flags}`, original error message: {err}"
      )),
    }?;

    Ok(Self(regex))
  }
}
