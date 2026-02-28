module.exports = function () {
	return "abc";
};

exports = module.exports;

module.exports.test = function () {
	return exports();
};
