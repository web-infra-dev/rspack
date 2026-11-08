const path = require("path");

module.exports = function(source) {
	const dependency = path.join(path.dirname(this.resourcePath), "overlap-dependency.txt");
	this.addDependency(this.resourcePath);
	this.addDependency(path.join(path.dirname(this.resourcePath), "trigger.txt"));
	this.addDependency(dependency);
	if (this.getOptions().kind === "build") this.addBuildDependency(dependency);
	return source;
};
