async function __rspack_dynamic_require__(module_id, chunk_id) {
  return import(`./${chunk_id}`);
}

globalThis.rs.dynamic_require = globalThis.rs.dynamic_require || __rspack_dynamic_require__;
