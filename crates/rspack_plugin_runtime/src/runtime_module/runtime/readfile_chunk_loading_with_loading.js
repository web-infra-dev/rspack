// ReadFile + VM.run chunk loading for javascript"
__webpack_require__.f.readFileVm = function(chunkId, promises) {
  var installedChunkData = installedChunks[chunkId];
  if (installedChunkData !== 0) {  // 0 means "already installed".
    // array of [resolve, reject, promise] means "currently loading"
    if (installedChunkData) {
      promises.push(installedChunkData[2]);
    } else {
      if (JS_MATCHER) {  // all chunks have JS
        // load the chunk and return promise to it
        var promise = new Promise(function(resolve, reject) {
          installedChunkData = installedChunks[chunkId] = [resolve, reject];
          var filename = require('path').join(
              __dirname, '$OUTPUT_DIR$' + __webpack_require__.u(chunkId));
          require('fs').readFile(filename, 'utf-8', function(err, content) {
            if (err) return reject(err);
            var chunk = {};
            require('vm').runInThisContext(
                '(function(exports, require, __dirname, __filename) {' +
                    content + '\n})',
                filename)(
                chunk, require, require('path').dirname(filename), filename);
            installChunk(chunk);
          });
        });
        promises.push(installedChunkData[2] = promise);
      } else
        installedChunks[chunkId] = 0;
    }
  }
};
