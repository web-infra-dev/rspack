const path = require("path");

module.exports = function (content, sourceMap) {
	this.callback(null, content, sourceMap, {
		fromLoader1: path.basename(this.resourcePath)
	});
};
