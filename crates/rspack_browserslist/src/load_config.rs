use browserslist::Opts;

/// Configuration parsed from input string and context directory
#[derive(Debug, Default)]
pub struct BrowserslistHandlerConfig<'a> {
  /// Optional absolute path to config file
  pub config_path: Option<String>,
  /// Optional environment name
  pub env: Option<String>,
  /// Optional query string
  pub query: Option<String>,
  /// Context directory path, used for Browserslist Opts.path to locate config
  pub context: &'a str,
}

/// Parse input string and context directory (as &str) to extract config path, env, or query.
/// The context is stored in the returned struct for later use.
///
/// # Arguments
///
/// * `input` - Optional input string, e.g. absolute config path with optional env suffix, or query string
/// * `context` - Context directory as &str, used to set context path for config searching
///
/// # Returns
///
/// * `BrowserslistHandlerConfig` struct with parsed fields and context path
pub fn parse<'a>(input: Option<&str>, context: &'a str) -> BrowserslistHandlerConfig<'a> {
  let Some(input) = input else {
    return BrowserslistHandlerConfig {
      context,
      ..Default::default()
    };
  };

  // Regex pattern matches:
  // group 1: absolute path (optionally Windows drive letter)
  // group 2: optional env suffix after colon
  // same as JS: /^(?:((?:[A-Z]:)?[/\\].*?))?(?::(.+?))?$/i
  let re = regex::Regex::new(r"^(?:((?:[A-Z]:)?[/\\].*?))?(?::(.+?))?$")
    .expect("Should initialize browserlist regex");

  if let Some(caps) = re.captures(input) {
    let config_path = caps.get(1).map(|m| m.as_str().to_string());
    let env = caps.get(2).map(|m| m.as_str().to_string());

    if config_path.is_some() {
      return BrowserslistHandlerConfig {
        config_path,
        env,
        query: None,
        context,
      };
    }
  }

  // If input is not absolute path with optional env, treat as query string
  BrowserslistHandlerConfig {
    config_path: None,
    env: None,
    query: Some(input.to_string()),
    context,
  }
}

/// Loads the browsers list based on the input and context.
pub fn load_browserslist(input: Option<&str>, context: &str) -> Option<Vec<String>> {
  let BrowserslistHandlerConfig {
    config_path,
    env,
    query,
    context,
  } = parse(input, context);

  let mut opts = Opts::default();
  if let Some(config) = config_path {
    opts.config = Some(config);
  } else {
    opts.path = Some(context.to_string());
  }
  if let Some(e) = env {
    opts.env = Some(e);
  }

  match if let Some(q) = query {
    browserslist::resolve(vec![q], &opts)
  } else {
    // browserslist::execute only works on non-wasm targets
    #[cfg(target_arch = "wasm32")]
    {
      Ok(Vec::new())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
      browserslist::execute(&opts)
    }
  } {
    Ok(browsers) => Some(browsers.into_iter().map(|d| d.to_string()).collect()),
    Err(_) => None,
  }
}
