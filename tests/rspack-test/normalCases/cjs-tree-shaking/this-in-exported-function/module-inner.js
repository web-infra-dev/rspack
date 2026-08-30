exports.a = function () {
	const results = [];
	function inner() {
		results.push(this);
	}
	class Inner {
		method() {
			results.push(this);
		}
	}
	inner.call("inner");
	new Inner().method();
	return results.length;
};
exports.b = function () {
	return "b";
};
exports.usedExports = __webpack_exports_info__.usedExports;
