import { foo } from './reexport';

// should not panic at crates/rspack_plugin_javascript/src/dependency/esm/esm_import_dependency.rs
foo;