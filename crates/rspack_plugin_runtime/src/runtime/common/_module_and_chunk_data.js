// mount Modules
(function () {
	runtime.installedModules = __INSTALLED_MODULES__;
})();

// mount Chunks
(function () {
	runtime.installedChunks = {};
})();

// mount ModuleCache
(function () {
	runtime.moduleCache = {};
})();

(function () {
	runtime.createHot = function createHot(id) {
		const accepts = [];

		return {
			accepts,
			accept(ids, callback) {
				if (!ids) {
					accepts.push({
						ids: id,
						accpet: undefined
					});
				} else if (typeof ids === "function") {
					accepts.push({
						ids: id,
						accpet: ids
					});
				} else {
					accepts.push({
						ids,
						accpet: callback
					});
				}
			},
			dispose(callback) {
				accpets.push({
					id,
					dispose: callback
				});
			}
		};
	};
})();
