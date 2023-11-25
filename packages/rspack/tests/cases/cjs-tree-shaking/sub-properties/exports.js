exports.aaa = function () {
	return 1;
};
exports.aaa.bbb = function () {
	return 2;
};
Object.defineProperty(exports, "ccc", {
	value: function () {
		return 3;
	}
});
Object.defineProperty(exports.ccc, "ddd", {
	value: function () {
		return 4;
	}
});
