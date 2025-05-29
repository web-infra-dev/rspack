use lightningcss::targets::Browsers;

// code from https://github.com/parcel-bundler/lightningcss/blob/241df3a82b9779cb0df3bf01cd99871a2498405a/src/targets.rs#L48
pub fn browserslist_to_lightningcss_targets<S: AsRef<str>, I: IntoIterator<Item = S>>(
  query: I,
) -> Result<Option<Browsers>, browserslist::Error> {
  use browserslist::{resolve, Opts};

  from_distribs(resolve(query, &Opts::default())?)
}

fn parse_version(version: &str) -> Option<u32> {
  let version = version.split('-').next()?;
  let mut version = version.split('.');
  let major = version.next().and_then(|v| v.parse::<u32>().ok());
  if let Some(major) = major {
    let minor = version
      .next()
      .and_then(|v| v.parse::<u32>().ok())
      .unwrap_or(0);
    let patch = version
      .next()
      .and_then(|v| v.parse::<u32>().ok())
      .unwrap_or(0);
    let v: u32 = (major & 0xff) << 16 | (minor & 0xff) << 8 | (patch & 0xff);
    return Some(v);
  }

  None
}

fn from_distribs(
  distribs: Vec<browserslist::Distrib>,
) -> Result<Option<Browsers>, browserslist::Error> {
  let mut browsers = Browsers::default();
  let mut has_any = false;
  for distrib in distribs {
    macro_rules! browser {
      ($browser: ident) => {{
        if let Some(v) = parse_version(distrib.version()) {
          if browsers.$browser.is_none() || v < browsers.$browser.unwrap() {
            browsers.$browser = Some(v);
            has_any = true;
          }
        }
      }};
    }

    match distrib.name() {
      "android" => browser!(android),
      "chrome" | "and_chr" => browser!(chrome),
      "edge" => browser!(edge),
      "firefox" | "and_ff" => browser!(firefox),
      "ie" => browser!(ie),
      "ios_saf" => browser!(ios_saf),
      "opera" | "op_mob" => browser!(opera),
      "safari" => browser!(safari),
      "samsung" => browser!(samsung),
      _ => {}
    }
  }

  if !has_any {
    return Ok(None);
  }

  Ok(Some(browsers))
}
