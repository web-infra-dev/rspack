const fs = require("node:fs");
const path = require("node:path");
module.exports = function () {
	const input = path.join(this.rootContext, "generated.txt");
	this.addDependency(input);
	return `module.exports = ${JSON.stringify(fs.readFileSync(input, "utf8").trim())};`;
};
