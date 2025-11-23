const { reduce, sortBy } = require("lodash-es");

function mainExport() {
	return reduce([1, 2, 3], (sum, n) => sum + n, 0);
}

module.exports = mainExport;
module.exports.utilityA = function () {
	return "utility A";
};
module.exports.utilityB = function () {
	return "utility B";
};

module.exports = {
	...module.exports,
	spreadProperty: "spread value"
};
