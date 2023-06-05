use rspack_core::{BoxPlugin, ExternalItem, PluginExt};

use crate::ExternalPlugin;

pub enum ElectronTargetContext {
  Main,
  Preload,
  Renderer,
  None,
}

pub fn electron_target_plugin(context: ElectronTargetContext, plugins: &mut Vec<BoxPlugin>) {
  plugins.push(
    ExternalPlugin::new(
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
    )
    .boxed(),
  );
  match context {
    ElectronTargetContext::Main => plugins.push(
      ExternalPlugin::new(
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
      )
      .boxed(),
    ),
    ElectronTargetContext::Preload | ElectronTargetContext::Renderer => plugins.push(
      ExternalPlugin::new(
        "node-commonjs".to_string(),
        ["desktop-capturer", "ipc-renderer", "remote", "web-frame"]
          .into_iter()
          .map(|i| ExternalItem::String(i.to_string()))
          .collect(),
      )
      .boxed(),
    ),
    ElectronTargetContext::None => {}
  }
}
