use crate::regexp::JsRegExp;

pub trait JsRegExpExt {
  fn to_rspack_regex(&self) -> rspack_regex::RspackRegex;
}

impl JsRegExpExt for JsRegExp {
  fn to_rspack_regex(&self) -> rspack_regex::RspackRegex {
    let pat = self.source();
    let flags = self.flags();
    rspack_regex::RspackRegex::with_flags(&pat, &flags).unwrap_or_else(|_| {
      panic!(
        "Try convert {:?} to RspackRegex with flags: {:?} failed",
        pat, flags
      )
    })
  }
}
