use bitflags::bitflags;

use crate::CompilerOptions;

#[rspack_cacheable::cacheable]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct RuntimeGlobals(u128);

bitflags! {
  impl RuntimeGlobals: u128 {
    const REQUIRE_SCOPE = 1 << 0;

    /**
     * the internal module object
     */
    const MODULE = 1 << 1;

    /**
     * the internal module object
     */
    const MODULE_ID = 1 << 2;

    /**
     * the internal require function
     */
    const REQUIRE = 1 << 3;

    /**
     * the module cache
     */
    const MODULE_CACHE = 1 << 4;

    /**
     * the chunk ensure function
     */
    const ENSURE_CHUNK = 1 << 5;

    /**
     * an object with handlers to ensure a chunk
     */
    const ENSURE_CHUNK_HANDLERS = 1 << 6;

    /**
     * the bundle public path
     */
    const PUBLIC_PATH = 1 << 7;

    /**
     * the filename of the script part of the chunk
     */
    const GET_CHUNK_SCRIPT_FILENAME = 1 << 8;

    /**
     * the filename of the css part of the chunk
     */
    const GET_CHUNK_CSS_FILENAME = 1 << 9;

    /**
     * function to load a script tag.
     * Arguments: (url: string, done: (event) => void), key?: string | number, chunkId?: string | number) => void
     * done function is called when loading has finished or timeout occurred.
     * It will attach to existing script tags with data-webpack == uniqueName + ":" + key or src == url.
     */
    const LOAD_SCRIPT = 1 << 10;

    /**
     * the shorthand for Object.prototype.hasOwnProperty
     * using of it decreases the compiled bundle size
     */
    const HAS_OWN_PROPERTY = 1 << 11;

    /**
     * the module functions, with only write access
     */
    const MODULE_FACTORIES_ADD_ONLY = 1 << 12;

    /**
     * register deferred code, which will run when certain
     * chunks are loaded.
     * Signature: (chunkIds: Id[], fn: () => any, priority: int >= 0 = 0) => any
     * Returned value will be returned directly when all chunks are already loaded
     * When (priority & 1) it will wait for all other handlers with lower priority to
     * be executed before itself is executed
     */
    const ON_CHUNKS_LOADED = 1 << 13;

    /**
     * global callback functions for installing chunks
     */
    const CHUNK_CALLBACK = 1 << 14;

    /**
     * the module functions
     */
    const MODULE_FACTORIES = 1 << 15;

    /**
     * interceptor for module executions
     */
    const INTERCEPT_MODULE_EXECUTION = 1 << 16;

    /**
     * function downloading the update manifest
     */
    const HMR_DOWNLOAD_MANIFEST = 1 << 17;

    /**
     * array with handler functions to download chunk updates
     */
    const HMR_DOWNLOAD_UPDATE_HANDLERS = 1 << 18;

    const HMR_INVALIDATE_MODULE_HANDLERS = 1 << 19;

    /**
     * the filename of the HMR manifest
     */
    const GET_UPDATE_MANIFEST_FILENAME = 1 << 20;

    /**
     * the filename of the script part of the hot update chunk
     */
    const GET_CHUNK_UPDATE_SCRIPT_FILENAME = 1 << 21;

    /**
     * the filename of the css part of the hot update chunk
     */
    const GET_CHUNK_UPDATE_CSS_FILENAME = 1 << 22;

    /**
     * object with all hmr module data for all modules
     */
    const HMR_MODULE_DATA = 1 << 23;

    /**
     * the prefix for storing state of runtime modules when hmr is enabled
     */
    const HMR_RUNTIME_STATE_PREFIX = 1 << 24;

    /**
     * method to install a chunk that was loaded somehow
     * Signature: ({ id, ids, modules, runtime }) => void
     */
    const EXTERNAL_INSTALL_CHUNK = 1 << 25;

    /**
     * the webpack hash
     */
    const GET_FULL_HASH = 1 << 26;

    /**
     * the global object
     */
    const GLOBAL = 1 << 27;

    /**
     * runtime need to return the exports of the last entry module
     */
    const RETURN_EXPORTS_FROM_RUNTIME = 1 << 28;

    /**
     * instantiate a wasm instance from module exports object, id, hash and importsObject
     */
    const INSTANTIATE_WASM = 1 << 29;

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
    const ASYNC_MODULE = 1 << 30;

    /**
     * the baseURI of current document
     */
    const BASE_URI = 1 << 31;

    const MODULE_LOADED = 1 << 32;

    const STARTUP_ENTRYPOINT = 1 << 33;
    const STARTUP_CHUNK_DEPENDENCIES = 1 << 34;

    const CREATE_SCRIPT_URL = 1 << 35;

    const CREATE_SCRIPT = 1 << 36;

    const GET_TRUSTED_TYPES_POLICY = 1 << 37;

    const DEFINE_PROPERTY_GETTERS = 1 << 38;

    const ENTRY_MODULE_ID = 1 << 39;

    const STARTUP_NO_DEFAULT = 1 << 40;

    const ENSURE_CHUNK_INCLUDE_ENTRIES = 1 << 41;

    const STARTUP = 1 << 42;

    const MAKE_NAMESPACE_OBJECT = 1 << 43;

    const EXPORTS = 1 << 44;

    const COMPAT_GET_DEFAULT_EXPORT = 1 << 45;

    const CREATE_FAKE_NAMESPACE_OBJECT = 1 << 46;

    const NODE_MODULE_DECORATOR = 1 << 47;

    const ESM_MODULE_DECORATOR = 1 << 48;

    /**
     * the System.register context object
     */
    const SYSTEM_CONTEXT = 1 << 49;

    const THIS_AS_EXPORTS = 1 << 50;

    const CURRENT_REMOTE_GET_SCOPE = 1 << 51;

    const SHARE_SCOPE_MAP = 1 << 52;

    const INITIALIZE_SHARING = 1 << 53;

    const SCRIPT_NONCE = 1 << 54;

    const RELATIVE_URL = 1 << 55;

    const CHUNK_NAME = 1 << 56;

    const RUNTIME_ID = 1 << 57;

    // prefetch and preload
    const PREFETCH_CHUNK = 1 << 58;

    const PREFETCH_CHUNK_HANDLERS = 1 << 59;

    const PRELOAD_CHUNK = 1 << 60;

    const PRELOAD_CHUNK_HANDLERS = 1 << 61;

    const UNCAUGHT_ERROR_HANDLER = 1 << 62;

    // rspack only
    const RSPACK_VERSION = 1 << 63;

    const HAS_CSS_MODULES = 1 << 64;

    // rspack only
    const RSPACK_UNIQUE_ID = 1 << 65;

    const HAS_FETCH_PRIORITY = 1 << 66;

    // amd module support
    const AMD_DEFINE = 1 << 67;
    const AMD_OPTIONS = 1 << 68;

    // defer import support
    const ASYNC_MODULE_EXPORT_SYMBOL = 1 << 69;
    const MAKE_DEFERRED_NAMESPACE_OBJECT = 1 << 70;
    const MAKE_DEFERRED_NAMESPACE_OBJECT_SYMBOL = 1 << 71;

    // rspack only
    const ASYNC_FEDERATION_STARTUP = 1 << 72;
  }
}

impl Default for RuntimeGlobals {
  fn default() -> Self {
    Self::empty()
  }
}

pub fn runtime_globals_to_string(
  runtime_globals: &RuntimeGlobals,
  _compiler_options: &CompilerOptions,
) -> String {
  // TODO: use compiler options to get scope name
  let scope_name = "__webpack_require__";
  match *runtime_globals {
    RuntimeGlobals::REQUIRE_SCOPE => format!("{scope_name}.*"),
    RuntimeGlobals::MODULE => "module".to_string(),
    RuntimeGlobals::MODULE_ID => "module.id".to_string(),
    RuntimeGlobals::MODULE_LOADED => "module.loaded".to_string(),
    RuntimeGlobals::REQUIRE => scope_name.to_string(),
    RuntimeGlobals::MODULE_CACHE => format!("{scope_name}.c"),
    RuntimeGlobals::ENSURE_CHUNK => format!("{scope_name}.e"),
    RuntimeGlobals::ENSURE_CHUNK_HANDLERS => format!("{scope_name}.f"),
    RuntimeGlobals::PUBLIC_PATH => format!("{scope_name}.p"),
    RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME => format!("{scope_name}.u"),
    RuntimeGlobals::GET_CHUNK_CSS_FILENAME => format!("{scope_name}.k"),
    RuntimeGlobals::LOAD_SCRIPT => format!("{scope_name}.l"),
    RuntimeGlobals::HAS_OWN_PROPERTY => format!("{scope_name}.o"),
    RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY => format!("{scope_name}.m (add only)"),
    RuntimeGlobals::ON_CHUNKS_LOADED => format!("{scope_name}.O"),
    RuntimeGlobals::CHUNK_CALLBACK => "webpackChunk".to_string(),
    RuntimeGlobals::MODULE_FACTORIES => format!("{scope_name}.m"),
    RuntimeGlobals::INTERCEPT_MODULE_EXECUTION => format!("{scope_name}.i"),
    RuntimeGlobals::HMR_DOWNLOAD_MANIFEST => format!("{scope_name}.hmrM"),
    RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS => format!("{scope_name}.hmrC"),
    RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS => format!("{scope_name}.hmrI"),
    RuntimeGlobals::HMR_MODULE_DATA => format!("{scope_name}.hmrD"),
    RuntimeGlobals::HMR_RUNTIME_STATE_PREFIX => format!("{scope_name}.hmrS"),
    RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME => format!("{scope_name}.hmrF"),
    RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME => format!("{scope_name}.hu"),
    RuntimeGlobals::GET_CHUNK_UPDATE_CSS_FILENAME => format!("{scope_name}.hk"),
    RuntimeGlobals::AMD_DEFINE => format!("{scope_name}.amdD"),
    RuntimeGlobals::AMD_OPTIONS => format!("{scope_name}.amdO"),
    RuntimeGlobals::EXTERNAL_INSTALL_CHUNK => format!("{scope_name}.C"),
    RuntimeGlobals::GET_FULL_HASH => format!("{scope_name}.h"),
    RuntimeGlobals::GLOBAL => format!("{scope_name}.g"),
    RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME => "return-exports-from-runtime".to_string(),
    RuntimeGlobals::INSTANTIATE_WASM => format!("{scope_name}.v"),
    RuntimeGlobals::ASYNC_MODULE => format!("{scope_name}.a"),
    RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL => format!("{scope_name}.aE"),
    RuntimeGlobals::BASE_URI => format!("{scope_name}.b"),
    RuntimeGlobals::STARTUP_ENTRYPOINT => format!("{scope_name}.X"),
    RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES => format!("{scope_name}.x (chunk dependencies)"),
    RuntimeGlobals::CREATE_SCRIPT_URL => format!("{scope_name}.tu"),
    RuntimeGlobals::CREATE_SCRIPT => format!("{scope_name}.ts"),
    RuntimeGlobals::GET_TRUSTED_TYPES_POLICY => format!("{scope_name}.tt"),
    RuntimeGlobals::DEFINE_PROPERTY_GETTERS => format!("{scope_name}.d"),
    RuntimeGlobals::ENTRY_MODULE_ID => format!("{scope_name}.s"),
    RuntimeGlobals::STARTUP_NO_DEFAULT => format!("{scope_name}.x (no default handler)"),
    RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES => format!("{scope_name}.f (include entries)"),
    RuntimeGlobals::STARTUP => format!("{scope_name}.x"),
    RuntimeGlobals::MAKE_NAMESPACE_OBJECT => format!("{scope_name}.r"),
    RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT => format!("{scope_name}.z"),
    RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT_SYMBOL => format!("{scope_name}.zS"),
    RuntimeGlobals::EXPORTS => "__webpack_exports__".to_string(),
    RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT => format!("{scope_name}.n"),
    RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT => format!("{scope_name}.t"),
    RuntimeGlobals::ESM_MODULE_DECORATOR => format!("{scope_name}.hmd"),
    RuntimeGlobals::NODE_MODULE_DECORATOR => format!("{scope_name}.nmd"),
    RuntimeGlobals::SYSTEM_CONTEXT => format!("{scope_name}.y"),
    RuntimeGlobals::THIS_AS_EXPORTS => "top-level-this-exports".to_string(),
    RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE => format!("{scope_name}.R"),
    RuntimeGlobals::SHARE_SCOPE_MAP => format!("{scope_name}.S"),
    RuntimeGlobals::INITIALIZE_SHARING => format!("{scope_name}.I"),
    RuntimeGlobals::SCRIPT_NONCE => format!("{scope_name}.nc"),
    RuntimeGlobals::RELATIVE_URL => format!("{scope_name}.U"),
    RuntimeGlobals::CHUNK_NAME => format!("{scope_name}.cn"),
    RuntimeGlobals::RUNTIME_ID => format!("{scope_name}.j"),
    RuntimeGlobals::PREFETCH_CHUNK => format!("{scope_name}.E"),
    RuntimeGlobals::PREFETCH_CHUNK_HANDLERS => format!("{scope_name}.F"),
    RuntimeGlobals::PRELOAD_CHUNK => format!("{scope_name}.G"),
    RuntimeGlobals::PRELOAD_CHUNK_HANDLERS => format!("{scope_name}.H"),
    RuntimeGlobals::UNCAUGHT_ERROR_HANDLER => format!("{scope_name}.oe"),
    // rspack only
    RuntimeGlobals::RSPACK_VERSION => format!("{scope_name}.rv"),
    RuntimeGlobals::RSPACK_UNIQUE_ID => format!("{scope_name}.ruid"),
    RuntimeGlobals::HAS_CSS_MODULES => "has css modules".to_string(),
    RuntimeGlobals::ASYNC_FEDERATION_STARTUP => format!("{scope_name}.mfAsyncStartup"),
    RuntimeGlobals::HAS_FETCH_PRIORITY => "has fetch priority".to_string(),
    _ => unreachable!(),
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
