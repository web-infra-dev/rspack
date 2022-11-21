(function () {
	// no baseURI

	// object to store loaded chunks
	// "1" means "loaded", otherwise not loaded yet
	var installedChunks = INSTALLED_CHUNKS;

	var installChunk = function (chunk) {
		var moreModules = chunk.modules,
			chunkIds = chunk.ids,
			runtime = chunk.runtime;
		for (var moduleId in moreModules) {
			if (__webpack_require__.o(moreModules, moduleId)) {
				__webpack_require__.m[moduleId] = moreModules[moduleId];
			}
		}
		if (runtime) runtime(__webpack_require__);
		for (var i = 0; i < chunkIds.length; i++) installedChunks[chunkIds[i]] = 1;
	};

	// require() chunk loading for javascript
	__webpack_require__.f.require = (chunkId, promises) => {
		// "1" is the signal for "already loaded"
		if (!installedChunks[chunkId]) {
			if (JS_MATCHER) {
				installChunk(require("./" + __webpack_require__.u(chunkId)));
			} else installedChunks[chunkId] = 1;
		}
	};

	// module.exports = __webpack_require__;
	__webpack_require__.C = installChunk;

	// no HMR

	// no HMR manifest
})();
