__webpack_require__.f.j = function(chunkId, promises) {
    // import() chunk loading for javascript
    var installedChunkData = __webpack_require__.o(installedChunks, chunkId) ? installedChunks[chunkId] : undefined;
    if(installedChunkData !== 0) { // 0 means "already installed".'
        // a Promise means "currently loading".
        if(installedChunkData) {
            promises.push(installedChunkData[1]);
        } else {
            if (JS_MATCHER) {
                // setup Promise in chunk cache
                var promise = $importFunctionName$('$OUTPUT_DIR$' + __webpack_require__.u(chunkId)).then(installChunk, function(e){
                    if(installedChunks[chunkId] !== 0) installedChunks[chunkId] = undefined;
                        throw e;
                });
                var promise = Promise.race([promise, new Promise(function(resolve){
                    installedChunkData = installedChunks[chunkId] = [resolve];
                })]);
                promises.push(installedChunkData[1] = promise); 
            }
            else installedChunks[chunkId] = 0;
        }
    }
};   
