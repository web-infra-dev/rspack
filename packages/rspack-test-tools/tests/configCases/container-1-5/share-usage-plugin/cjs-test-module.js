const { map, groupBy, partition } = require("lodash-es");

function processData(data) {
	const mapped = map(data, x => x * 2);
	return mapped;
}

module.exports = {
	processData,
	helperFunction: function () {
		return "helper";
	}
};

module.exports.additionalExport = function () {
	return "additional";
};
