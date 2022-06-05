// { [chunkId]: [resolve, reject] }
var chunks = {};

function __rspack_jsonp_define_chunk__(chunk_id, load_modules) {
  var installedChunkData = chunks[chunk_id];

  if (installedChunkData) {
    // if load_module failed, the chunk will be failed to load
    try {
      load_modules();
    } catch (e) {
      installedChunkData[1](e);
    }
    // resolve the chunk after modules of the chunk have been installed.
    installedChunkData[0]();
  }
}

function __rspack_jsonp_dynamic_require__(module_id, chunk_id) {
  var installedChunkData;

  // chunk loading in progress or done.
  if (chunks[chunk_id]) return chunks[chunk_id][0].then(() => globalThis.rs.require(module_id));

  // initiate chunk loading
  var script = document.createElement('script');
  script.setAttribute('type', 'text/javascript');
  script.setAttribute('charset', 'utf-8');
  script.setAttribute('src', 'http://127.0.01:4444/' + chunk_id + '.js');
  script.setAttribute('rspack-chunk-id', chunk_id);

  var promise = new Promise((resolve, reject) => {
    installedChunkData = chunks[chunk_id] = [resolve, reject];
    script.onerror = reject;
  });

  document.body.appendChild(script);

  // require module after chunk is installed
  return promise.then(() => globalThis.rs.require(module_id));
}

globalThis.rs.define_chunk = globalThis.rs.define_chunk || __rspack_jsonp_define_chunk__;

globalThis.rs.dynamic_require = globalThis.rs.dynamic_require || __rspack_jsonp_dynamic_require__;
