const fs = require("node:fs");
const path = require("node:path");

module.exports = function () {
	const { input } = this.getOptions();
	if (!path.isAbsolute(input) || path.dirname(input) !== this.rootContext) throw new Error("Expected current in-root loader option");
	this.addDependency(input);
	fs.appendFileSync(path.join(this.rootContext, "build-counts"), "built\n");
	return `module.exports = ${JSON.stringify(fs.readFileSync(input, "utf8").trim())};`;
};
