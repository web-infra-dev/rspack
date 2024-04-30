module.exports = function () {
	return "abc";
};

module.exports.test = function () {
	return module.exports();
};
