var oldTags = [];
var newTags = [];
var applyHandler = function (options) {
	return {
		dispose: function () {},
		apply: function () {
			var moduleIds = [];
			newTags.forEach(function (info) {
				info[1].sheet.disabled = false;
			});
			while (oldTags.length) {
				var oldTag = oldTags.pop();
				if (oldTag.parentNode) oldTag.parentNode.removeChild(oldTag);
			}
			while (newTags.length) {
				var info = newTags.pop();
				// var chunkModuleIds = loadCssChunkData(__webpack_require__.m, info[1], info[0]);
				// chunkModuleIds.forEach(function(id) {
				//     moduleIds.push(id)
				// });
			}
			return moduleIds;
		}
	};
};
var cssTextKey = function (link) {
	return Array.from(link.sheet.cssRules, function (r) {
		return r.cssText
	}).join();
};
__webpack_require__.hmrC.css = function (
	chunkIds,
	removedChunks,
	removedModules,
	promises,
	applyHandlers,
	updatedModulesList
) {
	applyHandlers.push(applyHandler);
	chunkIds.forEach(function (chunkId) {
		var filename = __webpack_require__.k(chunkId);
		var url = __webpack_require__.p + filename;
		var oldTag = loadStylesheet(chunkId, url);
		if (!oldTag) return;
		promises.push(
			new Promise(function (resolve, reject) {
				var link = loadStylesheet(
					chunkId,
					url + (url.indexOf("?") < 0 ? "?" : "&") + "hmr=" + Date.now(),
					function (event) {
						if (event.type !== "load") {
							var error = new Error();
							var errorType = event && event.type;
							var realSrc = event && event.target && event.target.src;
							error.message =
								"Loading css hot update chunk " +
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
						} else {
							try {
								if (cssTextKey(oldTag) == cssTextKey(link)) {
									if (link.parentNode) link.parentNode.removeChild(link);
									return resolve();
								}
							} catch (e) {}
							// var factories = {};
							// loadCssChunkData(factories, link, chunkId);
							// Object.keys(factories).forEach(function(id) {
							//     (updatedModulesList.push(id));
							// });
							link.sheet.disabled = true;
							oldTags.push(oldTag);
							newTags.push([chunkId, link]);
							resolve();
						}
					},
					oldTag
				);
			})
		);
	});
};
