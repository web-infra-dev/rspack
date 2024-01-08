exports.abc = "abc";
exports.fn = function () {
	return "abc";
};

exports.test = function () {
	return exports.abc;
};

exports.whole = function () {
	return exports;
};

exports.func = function () {
	return exports.fn();
};
