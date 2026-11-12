const module = { value: 42 };

exports.shadow = module;
exports.load = function () {
	return new Promise(resolve => {
		require(["module"], resolve);
	});
};

exports.loadConstant = function () {
	return new Promise(resolve => {
		require("module".split(","), resolve);
	});
};
