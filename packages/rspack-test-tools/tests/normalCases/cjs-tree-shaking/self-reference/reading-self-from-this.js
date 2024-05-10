exports.abc = "abc";
exports.fn = function () {
	return "abc";
};

exports.test = () => {
	return this.abc;
};

exports.whole = () => {
	return this;
};

exports.func = () => {
	return this.fn();
};
