use rspack_core::{BoxPlugin, ExternalItem, PluginExt};

use crate::ExternalsPlugin;

#[derive(Debug)]
pub enum ElectronTargetContext {
  Main,
  Preload,
  Renderer,
  None,
}

impl From<String> for ElectronTargetContext {
  fn from(value: String) -> Self {
    match value.as_str() {
      "main" => Self::Main,
      "preload" => Self::Preload,
      "renderer" => Self::Renderer,
      "none" => Self::None,
      _ => {
        unreachable!("ElectronTargetContext should only be one of main, preload, renderer, none")
      }
    }
  }
}

pub fn electron_target_plugin(context: &ElectronTargetContext, plugins: &mut Vec<BoxPlugin>) {
  plugins.push(
    ExternalsPlugin::new(
      "node-commonjs".to_string(),
      [
        "clipboard",
        "crash-reporter",
        "electron",
        "ipc",
        "native-image",
        "original-fs",
        "screen",
        "shell",
      ]
      .into_iter()
      .map(|i| ExternalItem::String(i.to_string()))
      .collect(),
      false,
    )
    .boxed(),
  );
  match context {
    ElectronTargetContext::Main => plugins.push(
      ExternalsPlugin::new(
        "node-commonjs".to_string(),
        [
          "app",
          "auto-updater",
          "browser-window",
          "content-tracing",
          "dialog",
          "global-shortcut",
          "ipc-main",
          "menu",
          "menu-item",
          "power-monitor",
          "power-save-blocker",
          "protocol",
          "session",
          "tray",
          "web-contents",
        ]
        .into_iter()
        .map(|i| ExternalItem::String(i.to_string()))
        .collect(),
        false,
      )
      .boxed(),
    ),
    ElectronTargetContext::Preload | ElectronTargetContext::Renderer => plugins.push(
      ExternalsPlugin::new(
        "node-commonjs".to_string(),
        ["desktop-capturer", "ipc-renderer", "remote", "web-frame"]
          .into_iter()
          .map(|i| ExternalItem::String(i.to_string()))
          .collect(),
        false,
      )
      .boxed(),
    ),
    ElectronTargetContext::None => {}
  }
}
