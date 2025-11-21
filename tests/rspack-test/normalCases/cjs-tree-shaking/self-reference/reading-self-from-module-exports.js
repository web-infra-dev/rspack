exports.abc = "abc";
exports.fn = function () {
	return "abc";
};

exports.test = function () {
	return module.exports.abc;
};

exports.whole = function () {
	return module.exports;
};

exports.func = function () {
	return module.exports.fn();
};
