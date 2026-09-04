module.exports = {
	used: "used",
	unusedMethod() {
		return require("./heavy");
	},
	unusedFullRequire() {
		return require("./heavy").toUpperCase();
	},
	get unusedGetter() {
		return require("./heavy");
	},
	set unusedSetter(_value) {
		require("./heavy");
	},
	usedExports: __webpack_exports_info__.usedExports
};
