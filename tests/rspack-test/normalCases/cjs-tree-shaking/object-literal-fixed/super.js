module.exports = {
	used: "used",
	unusedMethod() {
		return super.toString();
	},
	get unusedGetter() {
		return super.valueOf();
	},
	usedExports: __webpack_exports_info__.usedExports
};
