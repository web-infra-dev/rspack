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

	// TODO: should use runtime generator
	//---- hot require
	try {
		var execOptions = {
			id: moduleId,
			module: module,
			factory: runtime.installedModules[moduleId],
			require: __rspack_require__
		};
		module = execOptions.module;
		__rspack_require__.i.forEach(function (handler) {
			handler(execOptions);
		});
		execOptions.factory.call(
			module.exports,
			module,
			module.exports,
			execOptions.require.bind(this),
			runtime.__rspack_dynamic_require__ &&
				runtime.__rspack_dynamic_require__.bind(this),
			runtime
		);
	} catch (error) {
		module.error = error;
		throw error;
	}

	//------ other
	// this.installedModules[moduleId](
	// 	module,
	// 	module.exports,
	// 	this.__rspack_require__.bind(this),
	// 	this.__rspack_dynamic_require__ &&
	// 		this.__rspack_dynamic_require__.bind(this),
	//  runtime,
	// );

	return module.exports;
}

// mount require function
(function () {
	runtime.__rspack_require__ = __rspack_require__;
	// module execution interceptor
	runtime.__rspack_require__.i = [];
	// hasOwnProperty shorthand
	runtime.__rspack_require__.o = (obj, prop) =>
		Object.prototype.hasOwnProperty.call(obj, prop);
})();
