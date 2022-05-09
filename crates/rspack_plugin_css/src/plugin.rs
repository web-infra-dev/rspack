use crate::handle_with_css::{is_css_source, CssSourceType};
use async_trait::async_trait;
use rspack_core::{Asset, BundleContext, BundleOptions, Chunk, NormalizedBundleOptions};
use rspack_core::{Plugin, PluginLoadHookOutput};
use rspack_style::new_less::applicationn::Application;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use tokio::runtime::{Handle, Runtime};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct CssSourcePlugin {
  pub css_source_collect: Mutex<Vec<CssSourceType>>,
}

pub static RSPACK_CSS_PLUGIN_NSME: &'static str = "rspack_css_plugin";

#[derive(Debug)]
pub struct CssReferenceInfo {
  pub source: String,
  pub ref_count: usize,
  pub filepath: String,
}

impl Default for CssSourcePlugin {
  fn default() -> Self {
    CssSourcePlugin {
      css_source_collect: Mutex::new(vec![]),
    }
  }
}

impl CssSourcePlugin {
  pub fn handle_with_css_file(filepath: &str) -> HashMap<String, String> {
    let app = Application::default();
    let map = match app.render_into_hashmap(filepath.to_string()) {
      Ok(map) => map,
      Err(_) => {
        panic!("parse css has failed")
      }
    };
    map
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
    RSPACK_CSS_PLUGIN_NSME
  }

  #[inline]
  async fn load(&self, _ctx: &BundleContext, filepath: &str) -> PluginLoadHookOutput {
    if let Some(mut css) = is_css_source(filepath) {
      {
        let map = Self::handle_with_css_file(filepath);
        css.source_content_map = Some(map);
      }
      let mut list = self.css_source_collect.lock().await;
      list.push(css.clone());
      Some(format!("//{}\n", filepath) + r#"export {}"#)
    } else {
      None
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
    for css_source in css_source_list.iter_mut() {
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
      css_content += format!("\n{}", item.source).as_str();
    }
    if !css_content.is_empty() {
      ctx.emit_asset(Asset {
        source: css_content,
        filename: bundle_options.outdir.clone() + format!("/{}.css", entry_name).as_str(),
      })
    }
  }
}
