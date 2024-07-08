use rspack_error::{error, Result};
pub use swc_core::ecma::ast::EsVersion;

// TODO(swc-loader): Target still coupled with javascript downgrade, it should only affect runtime
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
  pub es_version: TargetEsVersion,
}

impl Target {
  pub fn new(args: &Vec<String>) -> Result<Target> {
    let mut es_version = TargetEsVersion::None;

    for item in args {
      let item = item.as_str();
      if item.starts_with("es") || item == "browserslist" {
        // es version
        if !es_version.is_none() {
          return Err(error!("Target es version conflict"));
        }
        let version = match item {
          "browserslist" => TargetEsVersion::BrowsersList,
          "es3" => TargetEsVersion::Esx(EsVersion::Es3),
          "es5" => TargetEsVersion::Esx(EsVersion::Es5),
          "es6" => TargetEsVersion::Esx(EsVersion::Es2015),
          "es2015" => TargetEsVersion::Esx(EsVersion::Es2015),
          "es2016" => TargetEsVersion::Esx(EsVersion::Es2016),
          "es2017" => TargetEsVersion::Esx(EsVersion::Es2017),
          "es2018" => TargetEsVersion::Esx(EsVersion::Es2018),
          "es2019" => TargetEsVersion::Esx(EsVersion::Es2019),
          "es2020" => TargetEsVersion::Esx(EsVersion::Es2020),
          "es2021" => TargetEsVersion::Esx(EsVersion::Es2021),
          "es2022" => TargetEsVersion::Esx(EsVersion::Es2022),
          _ => {
            return Err(error!("Unknown target es version {}", item));
          }
        };
        es_version = version;
        continue;
      }
    }

    Ok(Target { es_version })
  }
}
