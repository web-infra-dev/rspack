__webpack_require__.f = {};
// This file contains only the entry chunk.
// The chunk loading function for additional chunks
__webpack_require__.e = function (chunkId$FETCH_PRIORITY$) {
	return Promise.all(
		Object.keys(__webpack_require__.f).reduce(function (promises, key) {
			__webpack_require__.f[key](chunkId, promises$FETCH_PRIORITY$);
			return promises;
		}, [])
	);
};
