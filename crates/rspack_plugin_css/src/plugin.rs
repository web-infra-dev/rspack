use crate::handle_with_css::{is_css_source, CssSourceType};
use async_trait::async_trait;
use nodejs_resolver::{ResolveResult, Resolver};
use rspack_core::Plugin;
use rspack_core::{
  Asset, BundleContext, Chunk, Loader, NormalizedBundleOptions, PluginTransformHookOutput,
};
use rspack_style::new_less::applicationn::Application;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::runtime::{Handle, Runtime};

#[derive(Debug)]
pub struct CssSourcePlugin {
  pub css_source_collect: Mutex<Vec<CssSourceType>>,
  pub app: Application,
}

pub static PLUGIN_NAME: &'static str = "rspack_css_plugin";

#[derive(Debug)]
pub struct CssReferenceInfo {
  pub source: String,
  pub ref_count: usize,
  pub filepath: String,
}

impl Default for CssSourcePlugin {
  fn default() -> Self {
    let app = Application::default();

    let css_plugin = CssSourcePlugin {
      css_source_collect: Mutex::new(vec![]),
      app,
    };
    css_plugin
      .app
      .context
      .lock()
      .unwrap()
      .option
      .hooks
      .import_alias = Some(Arc::new(|filepath, importpath| {
      let resolver = Resolver::default()
        .with_extensions(vec!["less", "css", "scss", "sass"])
        .with_description_file(None);
      println!("{},{}", filepath, importpath);
      let res: ResolveResult = resolver.resolve(
        &Path::new(filepath.as_str()).parent().unwrap(),
        importpath.as_str(),
      )?;
      if let ResolveResult::Path(abs_path) = res {
        Ok(abs_path.to_str().unwrap().to_string())
      } else {
        Ok(filepath)
      }
    }));
    css_plugin
  }
}

impl CssSourcePlugin {
  pub fn handle_with_css_file(&self, filepath: &str) -> (HashMap<String, String>, String) {
    let res = match self.app.render_into_hashmap(filepath) {
      Ok(map) => map,
      Err(msg) => {
        println!("{}", msg);
        panic!("parse css has failed")
      }
    };
    res
  }

  pub fn get_runtime_handle() -> (Handle, Option<Runtime>) {
    match Handle::try_current() {
      Ok(h) => (h, None),
      Err(_) => {
        let rt = Runtime::new().unwrap();
        (rt.handle().clone(), Some(rt))
      }
    }
  }

  pub fn get_entry_name(entry_file_path: &str) -> String {
    let path = Path::new(entry_file_path);
    let entry_dir = path.parent().unwrap().to_str().unwrap().to_string();
    let ext = path
      .extension()
      .unwrap()
      .to_os_string()
      .into_string()
      .unwrap();
    entry_file_path
      .replace(&entry_dir, "")
      .replace(format!(".{}", ext).as_str(), "")
  }
}

#[async_trait]
impl Plugin for CssSourcePlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn transform(
    &self,
    _ctx: &BundleContext,
    uri: &str,
    loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    if let Some(Loader::Css) = loader {
      if let Some(mut css) = is_css_source(uri) {
        let mut js = format!("//{}\n", uri) + r#"export {}"#;
        {
          let (css_map, js_content) = self.handle_with_css_file(uri);
          css.source_content_map = Some(css_map);
          js = js_content;
        }
        let mut list = self.css_source_collect.lock().unwrap();
        list.push(css.clone());
        js
      } else {
        raw
      }
    } else {
      raw
    }
  }

  fn tap_generated_chunk(
    &self,
    ctx: &BundleContext,
    chunk: &Chunk,
    bundle_options: &NormalizedBundleOptions,
  ) {
    let mut css_content = "".to_string();
    let mut css_source_list = self.css_source_collect.try_lock().unwrap();
    let entry_name = Self::get_entry_name(chunk.id.as_str());

    let mut wait_sort_list: Vec<CssReferenceInfo> = vec![];
    for css_source in css_source_list
      .iter_mut()
      .filter(|x| chunk.module_ids.contains(&x.file_path))
    {
      for (filepath, source) in css_source.source_content_map.as_ref().unwrap() {
        if let Some(item) = wait_sort_list.iter_mut().find(|x| x.filepath == *filepath) {
          item.ref_count += 1;
        } else {
          wait_sort_list.push(CssReferenceInfo {
            source: source.to_string(),
            ref_count: 0,
            filepath: filepath.to_string(),
          })
        }
      }
    }
    wait_sort_list.sort_by(|x1, x2| x1.ref_count.cmp(&x2.ref_count));

    for item in wait_sort_list.iter().rev() {
      css_content += format!("{}", item.source).as_str();
    }

    if !css_content.is_empty() {
      ctx.emit_asset(Asset {
        source: css_content,
        filename: bundle_options.outdir.clone() + format!("/{}.css", entry_name).as_str(),
      })
    }
  }
}
