use anyhow::anyhow;
pub use swc_core::ecma::ast::EsVersion;

#[derive(Debug, Clone)]
pub enum TargetPlatform {
  BrowsersList,
  Web,
  WebWorker,
  Node(String),
  AsyncNode(String),
  None,
}

impl TargetPlatform {
  pub fn is_none(&self) -> bool {
    matches!(self, TargetPlatform::None)
  }
  pub fn is_web(&self) -> bool {
    matches!(self, TargetPlatform::BrowsersList | TargetPlatform::Web)
  }
  pub fn is_browsers_list(&self) -> bool {
    matches!(self, TargetPlatform::BrowsersList)
  }
}

#[derive(Debug, Clone)]
pub struct Target {
  pub platform: TargetPlatform,
  pub es_version: Option<EsVersion>,
}

impl Target {
  pub fn new(args: &Vec<String>) -> anyhow::Result<Target> {
    let mut platform: TargetPlatform = TargetPlatform::None;
    let mut es_version: Option<EsVersion> = None;

    for item in args {
      let item = item.as_str();
      if item.starts_with("es") {
        // es version
        if es_version.is_some() {
          return Err(anyhow!("Target es version conflict"));
        }
        let version = match item {
          "es3" => EsVersion::Es3,
          "es5" => EsVersion::Es5,
          "es2015" => EsVersion::Es2015,
          "es2016" => EsVersion::Es2016,
          "es2017" => EsVersion::Es2017,
          "es2018" => EsVersion::Es2018,
          "es2019" => EsVersion::Es2019,
          "es2020" => EsVersion::Es2020,
          "es2021" => EsVersion::Es2021,
          "es2022" => EsVersion::Es2022,
          _ => {
            return Err(anyhow!("Unknown target es version {}", item));
          }
        };
        es_version = Some(version);
        continue;
      }

      // platform
      if !platform.is_none() {
        return Err(anyhow!("Target platform conflict"));
      }
      platform = match item {
        "web" => TargetPlatform::Web,
        "webworker" => TargetPlatform::WebWorker,
        "browserslist" => TargetPlatform::BrowsersList,
        "node" => TargetPlatform::Node(String::new()),
        _ => {
          return Err(anyhow!("Unknown target platform {}", item));
        }
      };
    }

    Ok(Target {
      platform,
      es_version,
    })
  }
}
