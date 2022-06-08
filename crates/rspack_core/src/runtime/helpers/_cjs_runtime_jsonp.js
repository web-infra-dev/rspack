// { [chunkId]: [resolve, reject] }
var __rspack_chunks__ = {};

function loadStyles(url) {
  return new Promise((rsl, rej) => {
    var link = document.createElement('link');
    link.rel = 'stylesheet';
    link.type = 'text/css';
    link.href = url;
    link.onload = rsl;
    link.onerror = rej;
    var head = document.getElementsByTagName('head')[0];
    head.appendChild(link);
  });
}

function __rspack_jsonp_dynamic_require__(module_id, chunk_id) {
  var installedChunkData;

  // chunk loading in progress or done.
  if (__rspack_chunks__[chunk_id]) return __rspack_chunks__[chunk_id][2].then(() => globalThis.rs.require(module_id));

  // initiate chunk loading
  console.log('[rspack_jsonp] Loading chunk ' + chunk_id);
  var script = document.createElement('script');
  script.setAttribute('type', 'text/javascript');
  script.setAttribute('charset', 'utf-8');
  script.setAttribute('src', 'http://127.0.0.1:4444/' + chunk_id + '.js');
  script.setAttribute('rspack-chunk-id', chunk_id);

  var promise = new Promise((resolve, reject) => {
    installedChunkData = __rspack_chunks__[chunk_id] = [resolve, reject];
    script.onerror = reject;
  });

  installedChunkData.push(promise);

  document.body.appendChild(script);

  loadStyles('http://127.0.0.1:4444/' + chunk_id + '.css').catch((err) => {
    console.log('css load fail', err);
  });

  // require module after chunk is installed
  return promise.then(() => globalThis.rs.require(module_id));
}

globalThis.rs.dynamic_require = globalThis.rs.dynamic_require || __rspack_jsonp_dynamic_require__;

function __rspack_jsonp_define_chunk__(chunk_id, load_modules) {
  var installedChunkData = __rspack_chunks__[chunk_id];

  if (!installedChunkData) {
    // dummy chunk data for entry chunks
    installedChunkData = __rspack_chunks__[chunk_id] = [() => Promise.resolve(), null, Promise.resolve()];
  }

  // if load_module failed, the chunk will be failed to load too
  try {
    load_modules();
  } catch (e) {
    if (typeof installedChunkData[1] === 'function') {
      installedChunkData[1](e);
    } else {
      throw e;
    }
  }
  // resolve the chunk after modules of the chunk have been installed.
  installedChunkData[0]();
}

globalThis.rs.define_chunk = globalThis.rs.define_chunk || __rspack_jsonp_define_chunk__;
