module.exports.load = function () {
	return new Promise(resolve => {
		require("exports".split(","), resolve);
	});
};
