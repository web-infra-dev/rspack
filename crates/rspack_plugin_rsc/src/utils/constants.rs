use once_cell::sync::Lazy;
use regex::Regex;

pub static RSC_SERVER_ACTION_ENTRY_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"rsc-server-action-entry-loader").expect("regexp init failed"));
