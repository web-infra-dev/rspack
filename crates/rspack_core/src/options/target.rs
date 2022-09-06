use anyhow::anyhow;
use swc_ecma_ast::EsVersion;

#[derive(Debug, PartialEq, Eq)]
pub enum TargetPlatform {
  BrowsersList,
  Web,
  WebWorker,
  Node(String),
  None,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TargetEsVersion {
  Es(EsVersion),
  None,
}

#[derive(Debug)]
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
      if item.starts_with("es") {
        // es version
        if es_version != TargetEsVersion::None {
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
        es_version = TargetEsVersion::Es(version);
        continue;
      }

      // platform
      if platform != TargetPlatform::None {
        return Err(anyhow!("Target platform conflict"));
      }
      platform = match item {
        "web" => TargetPlatform::Web,
        "webworker" => TargetPlatform::WebWorker,
        "browserslist" => TargetPlatform::BrowsersList,
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
