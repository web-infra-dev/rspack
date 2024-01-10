__webpack_require__.f.consumes = function(chunkId, promises) {
	var moduleIdToConsumeDataMapping = __webpack_require__.consumesLoadingData.moduleIdToConsumeDataMapping
	var chunkMapping = __webpack_require__.consumesLoadingData.chunkMapping;
	if(__webpack_require__.o(chunkMapping, chunkId)) {
		chunkMapping[chunkId].forEach(function(id) {
			if(__webpack_require__.o(installedModules, id)) return promises.push(installedModules[id]);
			var onFactory = function(factory) {
				installedModules[id] = 0;
				__webpack_require__.m[id] = function(module) {
					delete __webpack_require__.c[id];
					module.exports = factory();
				}
			};
			var onError = function(error) {
				delete installedModules[id];
				__webpack_require__.m[id] = function(module) {
					delete __webpack_require__.c[id];
					throw error;
				}
			};
			try {
				var promise = resolveHandler(moduleIdToConsumeDataMapping[id])();
				if(promise.then) {
					promises.push(installedModules[id] = promise.then(onFactory)['catch'](onError));
				} else onFactory(promise);
			} catch(e) { onError(e); }
		});
	}
}
