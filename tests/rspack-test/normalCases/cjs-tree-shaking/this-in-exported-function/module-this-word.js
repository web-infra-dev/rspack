exports.b = function () {
	return "b";
};
exports.a = function (thisArg) {
	const _this = "this is a string";
	return typeof thisArg + _this.length;
};
exports.usedExports = __webpack_exports_info__.usedExports;
