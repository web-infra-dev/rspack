// require() chunk loading for javascript
__webpack_require__.f.require = function (chunkId, promises) {
	// "1" is the signal for "already loaded"
	if (!installedChunks[chunkId]) {
		if (JS_MATCHER) {
			installChunk(require("./" + __webpack_require__.u(chunkId)));
		} else installedChunks[chunkId] = 1;
	}
};
