exports.load = function () {
	return new Promise(resolve => {
		require(["module"], resolve);
	});
};

exports.loadMixed = function () {
	return new Promise(resolve => {
		require(["require", "module", "exports"], (loader, mod, value) => {
			resolve([loader, mod, value]);
		});
	});
};
