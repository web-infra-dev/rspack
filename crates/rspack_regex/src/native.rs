use std::fmt::Debug;

use cow_utils::CowUtils;
use regex::Regex;
use rspack_error::{Error, error};

#[derive(Debug, Clone)]
pub struct RspackNativeRegex(pub Regex);

impl RspackNativeRegex {
  pub fn test(&self, text: &str) -> bool {
    self.0.is_match(text)
  }

  pub fn with_flags(expr: &str, raw_flags: &str) -> Result<Self, Error> {
    let pattern = expr.cow_replace("\\/", "/");

    let mut flags = raw_flags.chars().collect::<Vec<char>>();
    flags.sort_unstable();
    let mut applied_flags = String::new();
    // https://github.com/vercel/next.js/blob/203adbd5d054609812d1f3666184875dcca13f3a/turbopack/crates/turbo-esregex/src/lib.rs#L71-L94
    for flag in &flags {
      match flag {
        // indices for substring matches: not relevant for the regex itself
        'd' => {}
        // global: default in rust, ignore
        'g' => {}
        // case-insensitive: letters match both upper and lower case
        'i' => applied_flags.push('i'),
        // multi-line mode: ^ and $ match begin/end of line
        'm' => applied_flags.push('m'),
        // allow . to match \n
        's' => applied_flags.push('s'),
        // Unicode support (enabled by default)
        'u' => applied_flags.push('u'),
        // sticky search: not relevant for the regex itself
        'y' => {}
        _ => {
          return Err(error!(
            "unsupported flag `{flag}` in regex: `{pattern}` with flags: `{raw_flags}`"
          ));
        }
      }
    }

    let regex = if applied_flags.is_empty() {
      Regex::new(&pattern).map_err(|e| error!(e))?
    } else {
      Regex::new(format!("(?{applied_flags}){pattern}").as_str()).map_err(|e| error!(e))?
    };

    Ok(Self(regex))
  }
}
