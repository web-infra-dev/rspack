exports.a = function () {
	const results = [];
	class Inner {
		static {
			results.push(this);
		}
		[`computed${1}`] = 1;
		field = this;
		method() {
			results.push(this);
		}
	}
	new Inner().method();
	return results.length;
};
exports.b = function () {
	return "b";
};
exports.usedExports = __webpack_exports_info__.usedExports;
