use std::sync::LazyLock;

use bitflags::bitflags;
use rustc_hash::FxHashMap;

use crate::CompilerOptions;

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct RuntimeGlobals(u128);

macro_rules! define_runtime_globals {
  (@step ($val:expr) ({$($acc:tt)*}) ($(#[$($attr:tt)+])* const $name:ident; $($rest:tt)*)) => {
    define_runtime_globals! {
      @step ($val << 1) ({
        $($acc)*
        $(#[$($attr)+])*
        const $name = $val;
      }) ($($rest)*)
    }
  };
  (@step ($val:expr) ({$($acc:tt)*}) ()) => {
    bitflags! {
      impl RuntimeGlobals: u128 {
        $($acc)*
      }
    }
  };
  ($($rest:tt)*) => {
    define_runtime_globals! {
      @step (1u128) ({}) ($($rest)*)
    }
  };
}

define_runtime_globals! {
  const REQUIRE_SCOPE;

  /**
   * the internal module object
   */
  const MODULE;

  /**
   * the internal module object
   */
  const MODULE_ID;

  /**
   * the internal require function
   */
  const REQUIRE;

  /**
   * the module cache
   */
  const MODULE_CACHE;

  /**
   * the chunk ensure function
   */
  const ENSURE_CHUNK;

  /**
   * an object with handlers to ensure a chunk
   */
  const ENSURE_CHUNK_HANDLERS;

  /**
   * the bundle public path
   */
  const PUBLIC_PATH;

  /**
   * the filename of the script part of the chunk
   */
  const GET_CHUNK_SCRIPT_FILENAME;

  /**
   * the filename of the css part of the chunk
   */
  const GET_CHUNK_CSS_FILENAME;

  /**
   * function to load a script tag.
   * Arguments: (url: string, done: (event) => void), key?: string | number, chunkId?: string | number) => void
   * done function is called when loading has finished or timeout occurred.
   * It will attach to existing script tags with data-webpack == uniqueName + ":" + key or src == url.
   */
  const LOAD_SCRIPT;

  /**
   * the shorthand for Object.prototype.hasOwnProperty
   * using of it decreases the compiled bundle size
   */
  const HAS_OWN_PROPERTY;

  /**
   * the module functions, with only write access
   */
  const MODULE_FACTORIES_ADD_ONLY;

  /**
   * register deferred code, which will run when certain
   * chunks are loaded.
   * Signature: (chunkIds: Id[], fn: () => any, priority: int >= 0 = 0) => any
   * Returned value will be returned directly when all chunks are already loaded
   * When (priority & 1) it will wait for all other handlers with lower priority to
   * be executed before itself is executed
   */
  const ON_CHUNKS_LOADED;

  /**
   * global callback functions for installing chunks
   */
  const CHUNK_CALLBACK;

  /**
   * the module functions
   */
  const MODULE_FACTORIES;

  /**
   * interceptor for module executions
   */
  const INTERCEPT_MODULE_EXECUTION;

  /**
   * function downloading the update manifest
   */
  const HMR_DOWNLOAD_MANIFEST;

  /**
   * array with handler functions to download chunk updates
   */
  const HMR_DOWNLOAD_UPDATE_HANDLERS;

  const HMR_INVALIDATE_MODULE_HANDLERS;

  /**
   * the filename of the HMR manifest
   */
  const GET_UPDATE_MANIFEST_FILENAME;

  /**
   * the filename of the script part of the hot update chunk
   */
  const GET_CHUNK_UPDATE_SCRIPT_FILENAME;

  /**
   * the filename of the css part of the hot update chunk
   */
  const GET_CHUNK_UPDATE_CSS_FILENAME;

  /**
   * object with all hmr module data for all modules
   */
  const HMR_MODULE_DATA;

  /**
   * the prefix for storing state of runtime modules when hmr is enabled
   */
  const HMR_RUNTIME_STATE_PREFIX;

  /**
   * method to install a chunk that was loaded somehow
   * Signature: ({ id, ids, modules, runtime }) => void
   */
  const EXTERNAL_INSTALL_CHUNK;

  /**
   * the webpack hash
   */
  const GET_FULL_HASH;

  /**
   * the global object
   */
  const GLOBAL;

  /**
   * runtime need to return the exports of the last entry module
   */
  const RETURN_EXPORTS_FROM_RUNTIME;

  /**
   * instantiate a wasm instance from module exports object, id, hash and importsObject
   */
  const INSTANTIATE_WASM;

  /**
   * Creates an async module. The body function must be a async function.
   * "module.exports" will be decorated with an AsyncModulePromise.
   * The body function will be called.
   * To handle async dependencies correctly do this: "([a, b, c] = await handleDependencies([a, b, c]));".
   * If "hasAwaitAfterDependencies" is truthy, "handleDependencies()" must be called at the end of the body function.
   * Signature: function(
   * module: Module,
   * body: (handleDependencies: (deps: AsyncModulePromise[]) => Promise<any[]> & () => void,
   * hasAwaitAfterDependencies?: boolean
   * ) => void
   */
  const ASYNC_MODULE;

  /**
   * the baseURI of current document
   */
  const BASE_URI;

  const MODULE_LOADED;

  const STARTUP_ENTRYPOINT;
  const STARTUP_CHUNK_DEPENDENCIES;

  const CREATE_SCRIPT_URL;

  const CREATE_SCRIPT;

  const GET_TRUSTED_TYPES_POLICY;

  const DEFINE_PROPERTY_GETTERS;

  const ENTRY_MODULE_ID;

  const STARTUP_NO_DEFAULT;

  const ENSURE_CHUNK_INCLUDE_ENTRIES;

  const STARTUP;

  const MAKE_NAMESPACE_OBJECT;

  const EXPORTS;

  const COMPAT_GET_DEFAULT_EXPORT;

  const CREATE_FAKE_NAMESPACE_OBJECT;

  const NODE_MODULE_DECORATOR;

  const ESM_MODULE_DECORATOR;

  /**
   * the System.register context object
   */
  const SYSTEM_CONTEXT;

  const THIS_AS_EXPORTS;

  const CURRENT_REMOTE_GET_SCOPE;

  const SHARE_SCOPE_MAP;

  const INITIALIZE_SHARING;

  const SCRIPT_NONCE;

  const RELATIVE_URL;

  const CHUNK_NAME;

  const RUNTIME_ID;

  // prefetch and preload
  const PREFETCH_CHUNK;

  const PREFETCH_CHUNK_HANDLERS;

  const PRELOAD_CHUNK;

  const PRELOAD_CHUNK_HANDLERS;

  const UNCAUGHT_ERROR_HANDLER;

  // rspack only
  const RSPACK_VERSION;

  const HAS_CSS_MODULES;

  // rspack only
  const RSPACK_UNIQUE_ID;

  const HAS_FETCH_PRIORITY;

  // amd module support
  const AMD_DEFINE;
  const AMD_OPTIONS;

  const TO_BINARY;

  // defer import support
  const ASYNC_MODULE_EXPORT_SYMBOL;
  const MAKE_DEFERRED_NAMESPACE_OBJECT;
  const MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT;
  const DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES;
  const DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES_SYMBOL;

  // rspack only
  const ASYNC_STARTUP;

  // react server component
  const RSC_MANIFEST;
}

impl Default for RuntimeGlobals {
  fn default() -> Self {
    Self::empty()
  }
}

pub static REQUIRE_SCOPE_GLOBALS: LazyLock<RuntimeGlobals> = LazyLock::new(|| {
  RuntimeGlobals::REQUIRE_SCOPE
    | RuntimeGlobals::MODULE_CACHE
    | RuntimeGlobals::ENSURE_CHUNK
    | RuntimeGlobals::ENSURE_CHUNK_HANDLERS
    | RuntimeGlobals::PUBLIC_PATH
    | RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME
    | RuntimeGlobals::GET_CHUNK_CSS_FILENAME
    | RuntimeGlobals::LOAD_SCRIPT
    | RuntimeGlobals::HAS_OWN_PROPERTY
    | RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY
    | RuntimeGlobals::ON_CHUNKS_LOADED
    | RuntimeGlobals::MODULE_FACTORIES
    | RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
    | RuntimeGlobals::HMR_DOWNLOAD_MANIFEST
    | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
    | RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS
    | RuntimeGlobals::HMR_MODULE_DATA
    | RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX
    | RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME
    | RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME
    | RuntimeGlobals::GET_CHUNK_UPDATE_CSS_FILENAME
    | RuntimeGlobals::AMD_DEFINE
    | RuntimeGlobals::AMD_OPTIONS
    | RuntimeGlobals::EXTERNAL_INSTALL_CHUNK
    | RuntimeGlobals::GET_FULL_HASH
    | RuntimeGlobals::GLOBAL
    | RuntimeGlobals::INSTANTIATE_WASM
    | RuntimeGlobals::ASYNC_MODULE
    | RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL
    | RuntimeGlobals::BASE_URI
    | RuntimeGlobals::STARTUP_ENTRYPOINT
    | RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES
    | RuntimeGlobals::CREATE_SCRIPT_URL
    | RuntimeGlobals::CREATE_SCRIPT
    | RuntimeGlobals::GET_TRUSTED_TYPES_POLICY
    | RuntimeGlobals::DEFINE_PROPERTY_GETTERS
    | RuntimeGlobals::ENTRY_MODULE_ID
    | RuntimeGlobals::STARTUP_NO_DEFAULT
    | RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES
    | RuntimeGlobals::STARTUP
    | RuntimeGlobals::MAKE_NAMESPACE_OBJECT
    | RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT
    | RuntimeGlobals::MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT
    | RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT
    | RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
    | RuntimeGlobals::ESM_MODULE_DECORATOR
    | RuntimeGlobals::NODE_MODULE_DECORATOR
    | RuntimeGlobals::SYSTEM_CONTEXT
    | RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE
    | RuntimeGlobals::SHARE_SCOPE_MAP
    | RuntimeGlobals::INITIALIZE_SHARING
    | RuntimeGlobals::SCRIPT_NONCE
    | RuntimeGlobals::RELATIVE_URL
    | RuntimeGlobals::CHUNK_NAME
    | RuntimeGlobals::RUNTIME_ID
    | RuntimeGlobals::PREFETCH_CHUNK
    | RuntimeGlobals::PREFETCH_CHUNK_HANDLERS
    | RuntimeGlobals::PRELOAD_CHUNK
    | RuntimeGlobals::PRELOAD_CHUNK_HANDLERS
    | RuntimeGlobals::UNCAUGHT_ERROR_HANDLER
    | RuntimeGlobals::RSPACK_VERSION
    | RuntimeGlobals::RSPACK_UNIQUE_ID
    | RuntimeGlobals::ASYNC_STARTUP
    | RuntimeGlobals::RSC_MANIFEST
    | RuntimeGlobals::TO_BINARY
    | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES
    | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES_SYMBOL
});

pub static MODULE_GLOBALS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| RuntimeGlobals::MODULE_ID | RuntimeGlobals::MODULE_LOADED);

pub fn runtime_globals_to_string(
  runtime_globals: &RuntimeGlobals,
  compiler_options: &CompilerOptions,
) -> String {
  if runtime_globals == &RuntimeGlobals::EXPORTS {
    return runtime_variable_to_string(&RuntimeVariable::Exports, compiler_options);
  }

  if runtime_globals == &RuntimeGlobals::REQUIRE {
    return runtime_variable_to_string(&RuntimeVariable::Require, compiler_options);
  }

  if runtime_globals == &RuntimeGlobals::MODULE {
    return "module".to_string();
  }

  let name = match *runtime_globals {
    RuntimeGlobals::REQUIRE_SCOPE => "*",
    RuntimeGlobals::MODULE_ID => "id",
    RuntimeGlobals::MODULE_LOADED => "loaded",
    RuntimeGlobals::MODULE_CACHE => "c",
    RuntimeGlobals::ENSURE_CHUNK => "e",
    RuntimeGlobals::ENSURE_CHUNK_HANDLERS => "f",
    RuntimeGlobals::PUBLIC_PATH => "p",
    RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME => "u",
    RuntimeGlobals::GET_CHUNK_CSS_FILENAME => "k",
    RuntimeGlobals::LOAD_SCRIPT => "l",
    RuntimeGlobals::HAS_OWN_PROPERTY => "o",
    RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY => "m (add only)",
    RuntimeGlobals::ON_CHUNKS_LOADED => "O",
    RuntimeGlobals::CHUNK_CALLBACK => "global chunk callback",
    RuntimeGlobals::MODULE_FACTORIES => "m",
    RuntimeGlobals::INTERCEPT_MODULE_EXECUTION => "i",
    RuntimeGlobals::HMR_DOWNLOAD_MANIFEST => "hmrM",
    RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS => "hmrC",
    RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS => "hmrI",
    RuntimeGlobals::HMR_MODULE_DATA => "hmrD",
    RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX => "hmrS",
    RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME => "hmrF",
    RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME => "hu",
    RuntimeGlobals::GET_CHUNK_UPDATE_CSS_FILENAME => "hk",
    RuntimeGlobals::AMD_DEFINE => "amdD",
    RuntimeGlobals::AMD_OPTIONS => "amdO",
    RuntimeGlobals::EXTERNAL_INSTALL_CHUNK => "C",
    RuntimeGlobals::GET_FULL_HASH => "h",
    RuntimeGlobals::GLOBAL => "g",
    RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME => "return-exports-from-runtime",
    RuntimeGlobals::INSTANTIATE_WASM => "v",
    RuntimeGlobals::ASYNC_MODULE => "a",
    RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL => "aE",
    RuntimeGlobals::BASE_URI => "b",
    RuntimeGlobals::STARTUP_ENTRYPOINT => "X",
    RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES => "x (chunk dependencies)",
    RuntimeGlobals::CREATE_SCRIPT_URL => "tu",
    RuntimeGlobals::CREATE_SCRIPT => "ts",
    RuntimeGlobals::GET_TRUSTED_TYPES_POLICY => "tt",
    RuntimeGlobals::DEFINE_PROPERTY_GETTERS => "d",
    RuntimeGlobals::ENTRY_MODULE_ID => "s",
    RuntimeGlobals::STARTUP_NO_DEFAULT => "x (no default handler)",
    RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES => "f (include entries)",
    RuntimeGlobals::STARTUP => "x",
    RuntimeGlobals::MAKE_NAMESPACE_OBJECT => "r",
    RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT => "z",
    RuntimeGlobals::MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT => "zO",
    RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES => "zT",
    RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES_SYMBOL => "zS",
    RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT => "n",
    RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT => "t",
    RuntimeGlobals::ESM_MODULE_DECORATOR => "hmd",
    RuntimeGlobals::NODE_MODULE_DECORATOR => "nmd",
    RuntimeGlobals::SYSTEM_CONTEXT => "y",
    RuntimeGlobals::THIS_AS_EXPORTS => "top-level-this-exports",
    RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE => "R",
    RuntimeGlobals::SHARE_SCOPE_MAP => "S",
    RuntimeGlobals::INITIALIZE_SHARING => "I",
    RuntimeGlobals::SCRIPT_NONCE => "nc",
    RuntimeGlobals::RELATIVE_URL => "U",
    RuntimeGlobals::CHUNK_NAME => "cn",
    RuntimeGlobals::RUNTIME_ID => "j",
    RuntimeGlobals::PREFETCH_CHUNK => "E",
    RuntimeGlobals::PREFETCH_CHUNK_HANDLERS => "F",
    RuntimeGlobals::PRELOAD_CHUNK => "G",
    RuntimeGlobals::PRELOAD_CHUNK_HANDLERS => "H",
    RuntimeGlobals::UNCAUGHT_ERROR_HANDLER => "oe",
    // rspack only
    RuntimeGlobals::RSPACK_VERSION => "rv",
    RuntimeGlobals::RSPACK_UNIQUE_ID => "ruid",
    RuntimeGlobals::HAS_CSS_MODULES => "has css modules",
    RuntimeGlobals::ASYNC_STARTUP => "asyncStartup",
    RuntimeGlobals::HAS_FETCH_PRIORITY => "has fetch priority",

    RuntimeGlobals::RSC_MANIFEST => "rscM",
    RuntimeGlobals::TO_BINARY => "tb",
    _ => unreachable!(),
  };
  if REQUIRE_SCOPE_GLOBALS.contains(*runtime_globals) {
    let require = runtime_variable_to_string(&RuntimeVariable::Require, compiler_options);
    return format!("{require}.{name}");
  }
  if MODULE_GLOBALS.contains(*runtime_globals) {
    let module = runtime_globals_to_string(&RuntimeGlobals::MODULE, compiler_options);
    return format!("{module}.{name}");
  }
  name.to_string()
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum RuntimeVariable {
  Require,
  Modules,
  ModuleCache,
  Module,
  Exports,
  StartupExec,
}

pub fn runtime_variable_to_string(
  runtime_variable: &RuntimeVariable,
  _compiler_options: &CompilerOptions,
) -> String {
  // TODO: use compiler options to get runtime variable names
  match *runtime_variable {
    RuntimeVariable::Require => "__webpack_require__".to_string(),
    RuntimeVariable::Modules => "__webpack_modules__".to_string(),
    RuntimeVariable::ModuleCache => "__webpack_module_cache__".to_string(),
    RuntimeVariable::Exports => "__webpack_exports__".to_string(),
    RuntimeVariable::Module => "__webpack_module__".to_string(),
    RuntimeVariable::StartupExec => "__webpack_exec__".to_string(),
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_iter() {
    let flags = RuntimeGlobals::PUBLIC_PATH | RuntimeGlobals::GET_CHUNK_CSS_FILENAME;
    let flags: Vec<_> = flags.iter().collect();
    assert_eq!(flags.len(), 2);
    assert_eq!(flags[0], RuntimeGlobals::PUBLIC_PATH);
    assert_eq!(flags[1], RuntimeGlobals::GET_CHUNK_CSS_FILENAME);
  }
}

type RuntimeGlobalMap = (
  FxHashMap<RuntimeGlobals, &'static str>,
  FxHashMap<&'static str, RuntimeGlobals>,
);

static RUNTIME_GLOBAL_MAP: LazyLock<RuntimeGlobalMap> = LazyLock::new(|| {
  let mut to_js_map = FxHashMap::default();
  let mut from_js_map = FxHashMap::default();

  for (name, value) in RuntimeGlobals::all().iter_names() {
    to_js_map.insert(value, name);
    from_js_map.insert(name, value);
  }

  to_js_map.shrink_to_fit();
  from_js_map.shrink_to_fit();
  (to_js_map, from_js_map)
});

impl RuntimeGlobals {
  pub fn to_names(&self) -> Vec<&'static str> {
    let mut res = vec![];

    for (item, js_name) in RUNTIME_GLOBAL_MAP.0.iter() {
      if self.contains(*item) {
        res.push(*js_name);
      }
    }
    res
  }
  pub fn from_names(names: &[String]) -> Self {
    let mut res = Self::empty();

    for name in names {
      if let Some(value) = RUNTIME_GLOBAL_MAP.1.get(name.as_str()) {
        res.insert(*value);
      }
    }
    res
  }
}
