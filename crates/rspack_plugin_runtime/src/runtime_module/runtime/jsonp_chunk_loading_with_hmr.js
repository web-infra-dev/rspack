var currentUpdatedModulesList;
var waitingUpdateResolves = {};
function loadUpdateChunk(chunkId, updatedModulesList) {
	currentUpdatedModulesList = updatedModulesList;
	return new Promise(function (resolve, reject) {
		// start update chunk loading
		var url = __webpack_require__.p + __webpack_require__.hu(chunkId);

		waitingUpdateResolves[chunkId] = resolve;
		// create error before stack unwound to get useful stacktrace later
		var error = new Error();
		var loadingEnded = function (event) {
			if (waitingUpdateResolves[chunkId]) {
				waitingUpdateResolves[chunkId] = undefined;
				var errorType =
					event && (event.type === "load" ? "missing" : event.type);
				var realSrc = event && event.target && event.target.src;
				error.message =
					"Loading hot update chunk " +
					chunkId +
					" failed.\n(" +
					errorType +
					": " +
					realSrc +
					")";
				error.name = "ChunkLoadError";
				error.type = errorType;
				error.request = realSrc;
				reject(error);
			}
		};
		__webpack_require__.l(url, loadingEnded);
	});
}

$globalObject$["hotUpdate"] = function (chunkId, moreModules, runtime) {
	for (var moduleId in moreModules) {
		if (__webpack_require__.o(moreModules, moduleId)) {
			currentUpdate[moduleId] = moreModules[moduleId];
			if (currentUpdatedModulesList) currentUpdatedModulesList.push(moduleId);
		}
	}
	if (runtime) currentUpdateRuntime.push(runtime);
	if (waitingUpdateResolves[chunkId]) {
		waitingUpdateResolves[chunkId]();
		waitingUpdateResolves[chunkId] = undefined;
	}
};
