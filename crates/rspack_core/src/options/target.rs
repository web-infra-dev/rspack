use anyhow::anyhow;
pub use swc_core::ecma::ast::EsVersion;

#[derive(Debug, Clone)]
pub enum TargetPlatform {
  Web,
  WebWorker,
  Node(String),
  None,
}

impl TargetPlatform {
  pub fn is_none(&self) -> bool {
    matches!(self, TargetPlatform::None)
  }
  pub fn is_web(&self) -> bool {
    matches!(self, TargetPlatform::Web)
  }
}

#[derive(Debug, Clone)]
pub enum TargetEsVersion {
  Esx(EsVersion),
  BrowsersList,
  None,
}

impl TargetEsVersion {
  pub fn is_none(&self) -> bool {
    matches!(self, TargetEsVersion::None)
  }
  pub fn is_browsers_list(&self) -> bool {
    matches!(self, TargetEsVersion::BrowsersList)
  }
}

#[derive(Debug, Clone)]
pub struct Target {
  pub platform: TargetPlatform,
  pub es_version: TargetEsVersion,
}

impl Target {
  pub fn new(args: &Vec<String>) -> anyhow::Result<Target> {
    let mut platform = TargetPlatform::None;
    let mut es_version = TargetEsVersion::None;

    for item in args {
      let item = item.as_str();
      if item.starts_with("es") || item == "browserslist" {
        // es version
        if !es_version.is_none() {
          return Err(anyhow!("Target es version conflict"));
        }
        let version = match item {
          "browserslist" => TargetEsVersion::BrowsersList,
          "es3" => TargetEsVersion::Esx(EsVersion::Es3),
          "es5" => TargetEsVersion::Esx(EsVersion::Es5),
          "es2015" => TargetEsVersion::Esx(EsVersion::Es2015),
          "es2016" => TargetEsVersion::Esx(EsVersion::Es2016),
          "es2017" => TargetEsVersion::Esx(EsVersion::Es2017),
          "es2018" => TargetEsVersion::Esx(EsVersion::Es2018),
          "es2019" => TargetEsVersion::Esx(EsVersion::Es2019),
          "es2020" => TargetEsVersion::Esx(EsVersion::Es2020),
          "es2021" => TargetEsVersion::Esx(EsVersion::Es2021),
          "es2022" => TargetEsVersion::Esx(EsVersion::Es2022),
          _ => {
            return Err(anyhow!("Unknown target es version {}", item));
          }
        };
        es_version = version;
        continue;
      }

      // platform
      if !platform.is_none() {
        return Err(anyhow!("Target platform conflict"));
      }
      platform = match item {
        "web" => TargetPlatform::Web,
        "webworker" => TargetPlatform::WebWorker,
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
