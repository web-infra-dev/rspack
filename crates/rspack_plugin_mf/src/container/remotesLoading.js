__webpack_require__.f.remotes = function (chunkId, promises) {
	var chunkMapping = __webpack_require__.remotesLoadingData.chunkMapping;
	var moduleIdToRemoteDataMapping = __webpack_require__.remotesLoadingData.moduleIdToRemoteDataMapping;
	if (__webpack_require__.o(chunkMapping, chunkId)) {
		chunkMapping[chunkId].forEach(function (id) {
			var getScope = __webpack_require__.R;
			if (!getScope) getScope = [];
			var data = moduleIdToRemoteDataMapping[id];
			if (getScope.indexOf(data) >= 0) return;
			getScope.push(data);
			if (data.p) return promises.push(data.p);
			var onError = function (error) {
				if (!error) error = new Error("Container missing");
				if (typeof error.message === "string")
					error.message +=
						'\nwhile loading "' + data.name + '" from ' + data.externalModuleId;
				__webpack_require__.m[id] = function () {
					throw error;
				};
				data.p = 0;
			};
			var handleFunction = function (fn, arg1, arg2, d, next, first) {
				try {
					var promise = fn(arg1, arg2);
					if (promise && promise.then) {
						var p = promise.then(function (result) {
							return next(result, d);
						}, onError);
						if (first) promises.push((data.p = p));
						else return p;
					} else {
						return next(promise, d, first);
					}
				} catch (error) {
					onError(error);
				}
			};
			var onExternal = function (external, _, first) {
				return external ? handleFunction(__webpack_require__.I, data.shareScope, 0, external, onInitialized, first) : onError();
			};
			var onInitialized = function (_, external, first) {
				return handleFunction(external.get, data.name, getScope, 0, onFactory, first);
			};
			var onFactory = function (factory) {
				data.p = 1;
				__webpack_require__.m[id] = function (module) {
					module.exports = factory();
				};
			};
			handleFunction(__webpack_require__, data.externalModuleId, 0, 0, onExternal, 1);
		});
	}
};
