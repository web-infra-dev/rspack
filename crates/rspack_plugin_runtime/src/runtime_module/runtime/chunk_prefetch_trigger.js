var chunkToChildrenMap = $CHUNK_MAP$;
__webpack_require__.f.prefetch = function (chunkId, promises) {
  Promise.all(promises).then(function () {
    var chunks = chunkToChildrenMap[chunkId];
    Array.isArray(chunks) && chunks.map(__webpack_require__.E);
  });
};