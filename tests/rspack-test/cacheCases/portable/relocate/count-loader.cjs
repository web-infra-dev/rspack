const fs = require("node:fs");
const path = require("node:path");

module.exports = function (source) {
	fs.appendFileSync(path.join(this.rootContext, "build-counts"), `${path.basename(this.resourcePath)}\n`);
	return source;
};
