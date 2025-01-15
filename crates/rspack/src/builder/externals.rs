#[derive(Debug, Default)]
pub struct ExternalsPresets {
  /// Treat node.js built-in modules like `fs`, `path` or `vm` as external and load them via `require()` when used.
  pub(crate) node: Option<bool>,

  /// Treat references to `http(s)://...` and `std:...` as external and load them via import when used.
  pub(crate) web: Option<bool>,

  /// Treat references to `http(s)://...` and `std:...` as external and load them via async import() when used
  pub(crate) web_async: Option<bool>,

  /// Treat common electron built-in modules in main and preload context like `electron`, `ipc` or `shell` as external and load them via `require()` when used.
  pub(crate) electron: Option<bool>,

  /// Treat electron built-in modules in the main context like `app`, `ipc-main` or `shell` as external and load them via `require()` when used.
  pub(crate) electron_main: Option<bool>,

  /// Treat electron built-in modules in the preload context like `web-frame`, `ipc-renderer` or `shell` as external and load them via require() when used.
  pub(crate) electron_preload: Option<bool>,

  /// Treat electron built-in modules in the preload context like `web-frame`, `ipc-renderer` or `shell` as external and load them via require() when used.
  pub(crate) electron_renderer: Option<bool>,

  /// Treat `NW.js` legacy `nw.gui` module as external and load it via `require()` when used.
  pub(crate) nwjs: Option<bool>,
}

impl ExternalsPresets {
  pub fn node(&self) -> bool {
    self.node.unwrap_or(false)
  }

  pub fn web(&self) -> bool {
    self.web.unwrap_or(false)
  }

  pub fn web_async(&self) -> bool {
    self.web_async.unwrap_or(false)
  }

  pub fn electron(&self) -> bool {
    self.electron.unwrap_or(false)
  }

  pub fn electron_main(&self) -> bool {
    self.electron_main.unwrap_or(false)
  }

  pub fn electron_preload(&self) -> bool {
    self.electron_preload.unwrap_or(false)
  }

  pub fn electron_renderer(&self) -> bool {
    self.electron_renderer.unwrap_or(false)
  }

  pub fn nwjs(&self) -> bool {
    self.nwjs.unwrap_or(false)
  }
}
