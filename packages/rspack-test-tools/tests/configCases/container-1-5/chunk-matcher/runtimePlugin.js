const runtimePlugin = function () {
    return {
        name: 'my-runtime-plugin',
        beforeInit(args) {

            const federationWarehouse = __webpack_require__.federation
            debugger;
            !function() {
                var installedChunks = {"115": 0,};
                var installChunk = function (chunk) {
                    var moreModules = chunk.modules, chunkIds = chunk.ids,
                        runtime = chunk.runtime;
                    for (var moduleId in moreModules) {
                        if (__webpack_require__.o(moreModules, moduleId)) {
                            __webpack_require__.m[moduleId] = moreModules[moduleId];
                        }
                    }
                    if (runtime) runtime(__webpack_require__);
                    for (var i = 0; i < chunkIds.length; i++) {
                        if (installedChunks[chunkIds[i]]) {
                            installedChunks[chunkIds[i]][0]();
                        }
                        installedChunks[chunkIds[i]] = 0;
                    }

                };

                // ReadFile + VM.run chunk loading for javascript"
                const handler = function (chunkId, promises) {
                    var installedChunkData = installedChunks[chunkId];
                    if (installedChunkData !== 0) {  // 0 means "already installed".
                        // array of [resolve, reject, promise] means "currently loading"
                        if (installedChunkData) {
                            promises.push(installedChunkData[2]);
                        } else {
                            if (__webpack_require__.federation.chunkMatcher(chunkId)) {  // all chunks have JS
                                // load the chunk and return promise to it
                                var promise

                                     promise = new Promise(function (resolve, reject) {
                                        installedChunkData = installedChunks[chunkId] = [resolve, reject];
                                        var filename = require('path').join(
                                            __dirname, "" + __webpack_require__.u(chunkId));
                                        require('fs').readFile(filename, 'utf-8', function (err, content) {
                                            if (err) return reject(err);
                                            var chunk = {};
                                            content = content.replace('__HANDLER__', 'PASS')
                                            require('vm').runInThisContext(
                                                '(function(exports, require, __dirname, __filename) {' +
                                                content + '\n})',
                                                filename)(
                                                chunk, require, require('path').dirname(filename), filename);
                                            installChunk(chunk);
                                        });
                                    });

                                promises.push(installedChunkData[2] = promise);
                            } else installedChunks[chunkId] = 0;

                        }
                    }

                };
                if(!__webpack_require__.f.j) {
                    __webpack_require__.f.j = handler
                } else {
                    __webpack_require__.f.readfileVm = handler
                }

            }();
            console.log('beforeInit: ', args);
            return args;
        },
        beforeRequest(args) {
            console.log('beforeRequest: ', args);
            return args;
        },
        afterResolve(args) {
            console.log('afterResolve', args);
            return args;
        },
        onLoad(args) {
            console.log('onLoad: ', args);
            return args;
        },
        async loadShare(args) {
            console.log('loadShare:', args);
        },
        async beforeLoadShare(args) {
            console.log('beforeloadShare:', args);
            return args;
        },
    };
};
export default runtimePlugin;
