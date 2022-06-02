use crate::handle_with_css::{is_style_source, StyleSourceType};
use async_trait::async_trait;
use nodejs_resolver::{ResolveResult, Resolver};
use rspack_core::{
  Asset, BundleContext, Chunk, Loader, NormalizedBundleOptions, PluginTransformHookOutput,
};
use rspack_core::{Plugin, PluginBuildStartHookOutput, PluginTapGeneratedChunkHookOutput};
use rspack_style::style_core::applicationn::Application;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::sync::{Arc, Mutex};

///
/// 样式插件
/// 处理 rspack 中 scss | sass | less | css
/// 设计原因: (需要拿到 在 js中 引用 css | less |sass  中 import 所有分页信息)
/// 并且在 tap_generated_chunk 聚合中  进行 tree-shaking
/// 如果 单独 处理 上述样式 的插件上下文 则 样式聚会 tree-shaking 可能会有问题!
///
#[derive(Debug)]
pub struct StyleSourcePlugin {
  pub style_source_collect: Mutex<Vec<StyleSourceType>>,
  pub app: Application,
}

pub static PLUGIN_NAME: &str = "rspack_style_source_plugin";

#[derive(Debug)]
pub struct SytleReferenceInfo {
  pub source: String,
  pub ref_count: usize,
  pub filepath: String,
}

impl Default for StyleSourcePlugin {
  fn default() -> Self {
    let app = Application::default();

    let style_plugin = Self {
      style_source_collect: Mutex::new(vec![]),
      app,
    };
    style_plugin
      .app
      .context
      .lock()
      .unwrap()
      .option
      .hooks
      .content_interceptor = None;
    style_plugin
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
        Path::new(filepath.as_str()).parent().unwrap(),
        importpath.as_str(),
      )?;
      if let ResolveResult::Path(abs_path) = res {
        Ok(abs_path.to_str().unwrap().to_string())
      } else {
        Ok(filepath)
      }
    }));
    style_plugin
  }
}

impl StyleSourcePlugin {
  ///
  /// 处理 css 文件
  /// 目前 在 ipc 保留的时候 同样的处理方式
  ///
  pub fn handle_with_css_file(
    &self,
    content: &str,
    filepath: &str,
  ) -> (HashMap<String, String>, String) {
    let res = match self
      .app
      .render_content_into_hashmap_with_css(content, filepath)
    {
      Ok(map) => map,
      Err(msg) => {
        println!("{}", msg);
        panic!("parse css has failed")
      }
    };
    res
  }

  ///
  /// 处理 less 文件
  /// 目前 在 ipc 保留的时候 同样的处理方式
  ///
  pub fn handle_with_less_file(&self, filepath: &str) -> (HashMap<String, String>, String) {
    let res = match self.app.render_into_hashmap(filepath) {
      Ok(map) => map,
      Err(msg) => {
        println!("{}", msg);
        panic!("parse css has failed")
      }
    };
    res
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
impl Plugin for StyleSourcePlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  #[inline]
  fn need_build_end(&self) -> bool {
    false
  }

  #[inline]
  fn need_resolve(&self) -> bool {
    false
  }

  #[inline]
  fn need_load(&self) -> bool {
    false
  }

  #[inline]
  fn need_transform_ast(&self) -> bool {
    false
  }

  ///
  /// 初始化参数
  ///
  async fn build_start(&self, ctx: &BundleContext) -> PluginBuildStartHookOutput {
    let minify = ctx.options.minify;
    self.app.set_minify(minify);
    Ok(())
  }

  fn transform(
    &self,
    _ctx: &BundleContext,
    uri: &str,
    loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    if let Some(Loader::Less) = loader {
      if let Some(mut style) = is_style_source(uri) {
        let js;
        {
          let (css_map, js_content) = self.handle_with_less_file(uri);
          style.source_content_map = Some(css_map);
          js = js_content;
        }
        let mut list = self
          .style_source_collect
          .lock()
          .map_err(|_| anyhow::anyhow!("failed to lock style_source_collect"))?;
        list.push(style);
        Ok(js)
      } else {
        // todo fix 这里应该报错 is_style_source 会检查文件的 绝对路径资源是否 符合 style 如果没有进来
        // todo 默认是 *.ts *.wasm 这种给了 Loader::less 而不是返回该内容 但 PluginTransformHookOutput 非 Result
        Ok(raw)
      }
    } else if let Some(Loader::Css) = loader {
      if let Some(mut style) = is_style_source(uri) {
        let js;
        {
          let (css_map, js_content) = self.handle_with_css_file(raw.as_str(), uri);
          style.source_content_map = Some(css_map);
          js = js_content;
        }
        let mut list = self
          .style_source_collect
          .lock()
          .map_err(|_| anyhow::anyhow!("failed to lock style_source_collect"))?;
        list.push(style);
        Ok(js)
      } else {
        Ok(raw)
      }
    } else if let Some(Loader::Sass) = loader {
      unimplemented!()
    } else {
      Ok(raw)
    }
  }

  ///
  /// 针对 css | sass | scss | less
  /// 统一 在上下文中 style_source_collect
  /// 注意 该内容 格式基本 为 css 文件名不同
  /// 所有的 tree-shaking 是基于文件名(绝对路径) 来进行处理
  /// example -> index.module.less index.module.css 必须视为两个文件!
  /// 上述 -> 也可以同时 指定 Loader:Css 处理 在 js-plguin 中完成降级
  ///
  fn tap_generated_chunk(
    &self,
    ctx: &BundleContext,
    chunk: &Chunk,
    bundle_options: &NormalizedBundleOptions,
  ) -> PluginTapGeneratedChunkHookOutput {
    let mut css_content = "".to_string();
    let mut css_source_list = self
      .style_source_collect
      .try_lock()
      .map_err(|_| anyhow::anyhow!("failed to lock style_source_collect"))?;

    let file_name = match chunk.filename.as_deref() {
      Some(filename) => filename,
      None => {
        anyhow::bail!("failed to get entry name")
      }
    };

    let entry_name = Self::get_entry_name(file_name);

    let mut wait_sort_list: Vec<SytleReferenceInfo> = vec![];
    for css_source in css_source_list
      .iter_mut()
      .filter(|x| chunk.module_uris.contains(&x.file_path))
    {
      for (filepath, source) in css_source
        .source_content_map
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("failed to get source_content_map"))?
      {
        if let Some(item) = wait_sort_list.iter_mut().find(|x| x.filepath == *filepath) {
          item.ref_count += 1;
        } else {
          wait_sort_list.push(SytleReferenceInfo {
            source: source.to_string(),
            ref_count: 0,
            filepath: filepath.to_string(),
          })
        }
      }
    }
    wait_sort_list.sort_by(|x1, x2| x1.ref_count.cmp(&x2.ref_count));

    for item in wait_sort_list.iter().rev() {
      css_content += item.source.to_string().as_str();
    }

    if !css_content.is_empty() {
      ctx.emit_asset(Asset {
        source: css_content,
        filename: bundle_options.outdir.clone() + format!("/{}.css", entry_name).as_str(),
      })
    }

    Ok(())
  }
}
