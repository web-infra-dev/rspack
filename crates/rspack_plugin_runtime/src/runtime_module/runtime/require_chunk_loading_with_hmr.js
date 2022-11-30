function loadUpdateChunk(chunkId, updatedModulesList) {
	var update = require("./" + __webpack_require__.hu(chunkId));
	var updatedModules = update.modules;
	var runtime = update.runtime;
	for (var moduleId in updatedModules) {
		if (__webpack_require__.o(updatedModules, moduleId)) {
			currentUpdate[moduleId] = updatedModules[moduleId];
			if (updatedModulesList) updatedModulesList.push(moduleId);
		}
	}
	if (runtime) currentUpdateRuntime.push(runtime);
}
