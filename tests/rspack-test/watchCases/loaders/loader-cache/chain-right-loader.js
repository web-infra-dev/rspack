const path = require("path");

module.exports = function (source) {
	this.addDependency(path.join(path.dirname(this.resourcePath), "chain-right.txt"));
	return source;
};
