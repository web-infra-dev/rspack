const cachedModule = __webpack_require__.c[module.id];

module.exports = {
	currentModuleCached: cachedModule === module,
	exportsShared: cachedModule.exports === exports,
};
