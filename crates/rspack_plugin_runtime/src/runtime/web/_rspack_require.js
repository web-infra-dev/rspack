// The require function
function __rspack_require__(moduleId) {
	var cachedModule = this.moduleCache[moduleId];
	if (cachedModule !== undefined) {
		return cachedModule.exports;
	}

	// Create a new module (and put it into the cache)
	var module = (this.moduleCache[moduleId] = {
		// no module.id needed
		// no module.loaded needed
		exports: {}
	});

	this.installedModules[moduleId](
		module,
		module.exports,
		this.__rspack_require__.bind(this),
		this.__rspack_dynamic_require__ &&
			this.__rspack_dynamic_require__.bind(this),
		runtime
	);

	return module.exports;
}

// mount require function
(function () {
	runtime.__rspack_require__ = __rspack_require__;
	// module execution interceptor
	runtime.__rspack_require__.i = [];
})();
