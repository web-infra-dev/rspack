const fs = require("fs");
const path = require("path");
const runs = new Map();

module.exports = function() {
	const dependency = path.join(path.dirname(this.resourcePath), "overlap-dependency.txt");
	if (this.getOptions().kind === "build") this.addBuildDependency(dependency);
	else this.addDependency(dependency);
	const count = (runs.get(this.resource) || 0) + 1;
	runs.set(this.resource, count);
	return `module.exports = ${JSON.stringify({ value: fs.readFileSync(dependency, "utf-8").trim(), runs: count })};`;
};
