__webpack_require__.X = function(result, chunkIds, fn) {
    // arguments: chunkIds, moduleId are deprecated
    var moduleId = chunkIds;
    if(!fn) {
        chunkIds = result;
        fn = function() {
            return __webpack_require__(__webpack_require__.s = moduleId);
        }
    }
    return Promise.all(chunkIds.map(__webpack_require__.e, __webpack_require__)).then(function () {
        var r = fn();
        return r === undefined ? result : r;
    });
}