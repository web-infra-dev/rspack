use std::sync::LazyLock;

static RSPACK_HOT_TEST: LazyLock<String> =
  LazyLock::new(|| std::env::var("RSPACK_HOT_TEST").ok().unwrap_or_default());

pub fn is_hot_test() -> bool {
  *RSPACK_HOT_TEST == "true"
}

pub static HOT_TEST_DEFINE_GLOBAL: LazyLock<String> = LazyLock::new(|| {
  if is_hot_test() {
    {
      r#"
      self.__HMR_UPDATED_RUNTIME__ = {
        javascript: {
          outdatedModules: [],
          outdatedDependencies: [],

          acceptedModules: [],
          updatedModules: [],
          updatedRuntime: [],
          disposedModules: [],
        },
        statusPath: []
      };
      "#
      .to_string()
    }
  } else {
    Default::default()
  }
});

pub static HOT_TEST_STATUS_CHANGE: LazyLock<String> = LazyLock::new(|| {
  if is_hot_test() {
    {
      r#"
      if (self.__HMR_UPDATED_RUNTIME__) {
        self.__HMR_UPDATED_RUNTIME__.statusPath.push(newStatus);
      }
      "#
      .to_string()
    }
  } else {
    Default::default()
  }
});

pub static HOT_TEST_OUTDATED: LazyLock<String> = LazyLock::new(|| {
  if is_hot_test() {
    {
      r#"
      if (self.__HMR_UPDATED_RUNTIME__) {
        self.__HMR_UPDATED_RUNTIME__.javascript.outdatedModules = outdatedModules;
        self.__HMR_UPDATED_RUNTIME__.javascript.outdatedDependencies = outdatedDependencies;
      }
      "#
      .to_string()
    }
  } else {
    Default::default()
  }
});

pub static HOT_TEST_DISPOSE: LazyLock<String> = LazyLock::new(|| {
  if is_hot_test() {
    {
      r#"
      if (disposeHandlers.length > 0 && self.__HMR_UPDATED_RUNTIME__) {
        self.__HMR_UPDATED_RUNTIME__.javascript.disposedModules.push(moduleId);
      }
      "#
      .to_string()
    }
  } else {
    Default::default()
  }
});

pub static HOT_TEST_UPDATED: LazyLock<String> = LazyLock::new(|| {
  if is_hot_test() {
    {
      r#"
      if (self.__HMR_UPDATED_RUNTIME__) {
        self.__HMR_UPDATED_RUNTIME__.javascript.updatedModules.push(updateModuleId);
      }
      "#
      .to_string()
    }
  } else {
    Default::default()
  }
});

pub static HOT_TEST_RUNTIME: LazyLock<String> = LazyLock::new(|| {
  if is_hot_test() {
    r#"
      currentUpdateRuntime[i](new Proxy(__webpack_require__, {
        set(target, prop, value, receiver) {
          if (self.__HMR_UPDATED_RUNTIME__) {
            self.__HMR_UPDATED_RUNTIME__.javascript.updatedRuntime.push(`__webpack_require__.${prop}`);
          }
          return Reflect.set(target, prop, value, receiver);
        }
      }));
      "#
      .to_string()
  } else {
    "currentUpdateRuntime[i](__webpack_require__);".to_string()
  }
});

pub static HOT_TEST_ACCEPT: LazyLock<String> = LazyLock::new(|| {
  if is_hot_test() {
    {
      r#"
      if (self.__HMR_UPDATED_RUNTIME__) {
        self.__HMR_UPDATED_RUNTIME__.javascript.acceptedModules.push(dependency);
      }
      "#
      .to_string()
    }
  } else {
    Default::default()
  }
});
