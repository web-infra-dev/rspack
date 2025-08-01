use rspack_error::{Result, error_bail};

#[derive(Debug)]
pub struct UrlAndGlobal<'a> {
  pub url: &'a str,
  pub global: &'a str,
}

pub fn extract_url_and_global(value: &str) -> Result<UrlAndGlobal<'_>> {
  let index = value.find('@');
  if let Some(index) = index
    && index != 0
  {
    return Ok(UrlAndGlobal {
      url: &value[index + 1..],
      global: &value[0..index],
    });
  }
  error_bail!("Invalid request \"{}\"", value)
}
