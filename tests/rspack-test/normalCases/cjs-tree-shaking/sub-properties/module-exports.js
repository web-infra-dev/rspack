module.exports.aaa = function () {
	return 1;
};
module.exports.aaa.bbb = function () {
	return 2;
};
Object.defineProperty(module.exports, "ccc", {
	value: function () {
		return 3;
	}
});
Object.defineProperty(module.exports.ccc, "ddd", {
	value: function () {
		return 4;
	}
});
