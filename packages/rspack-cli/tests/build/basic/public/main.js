(() => {
	// webpackBootstrap
	var __webpack_modules__ = {};
	/************************************************************************/
	// The module cache
	var __webpack_module_cache__ = {};

	// The require function
	function __webpack_require__(moduleId) {
		// Check if module is in cache
		var cachedModule = __webpack_module_cache__[moduleId];
		if (cachedModule !== undefined) {
			return cachedModule.exports;
		}
		// Create a new module (and put it into the cache)
		var module = (__webpack_module_cache__[moduleId] = {
			exports: {}
		});
		// Execute the module function
		__webpack_modules__[moduleId](module, module.exports, __webpack_require__);

		// Return the exports of the module
		return module.exports;
	}

	/************************************************************************/
	// webpack/runtime/rspack_version
	(() => {
		__webpack_require__.rv = function () {
			return "0.7.4";
		};
	})();
	// webpack/runtime/rspack_unique_id
	(() => {
		__webpack_require__.ruid = "bundler=rspack@0.7.4";
	})();
	/************************************************************************/
	var __webpack_exports__ = {};
	console.log("CONFIG");
})();
//# sourceMappingURL=main.js.map
