const path = require("path");

module.exports = function (content) {
	this.addContextDependency(path.resolve(__dirname, "./lib"));
	return content;
};
