exports.a = function () {
	return (() => this.b())();
};

exports.b = function () {
	return "b";
};

exports.usedExports = __webpack_exports_info__.usedExports;
