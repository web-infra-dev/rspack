// CommonJS module using exports.X = ... pattern

exports.formatDate = function (date) {
	return date.toISOString();
};

exports.processData = function (data) {
	return data.map(x => x * 2);
};

exports.unusedFunction = function () {
	return "This function is not imported";
};

exports.helperUtil = function () {
	return "Helper utility";
};

// Also test module.exports.X pattern
module.exports.parseJSON = function (str) {
	return JSON.parse(str);
};

module.exports.anotherUnused = function () {
	return "Another unused export";
};
