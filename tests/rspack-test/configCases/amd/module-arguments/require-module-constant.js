exports.load = function () {
	return new Promise(resolve => {
		require("module".split(","), resolve);
	});
};

exports.loadMixed = function () {
	return new Promise(resolve => {
		require("require,module,exports".split(","), (loader, mod, value) => {
			resolve([loader, mod, value]);
		});
	});
};
