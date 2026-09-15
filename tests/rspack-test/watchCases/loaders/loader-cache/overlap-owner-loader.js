const fs = require("fs");
const path = require("path");

module.exports = function (source) {
	const dirname = path.dirname(this.resourcePath);
	const owner = path.join(dirname, "overlap-owner.txt");
	this.addDependency(owner);
	if (fs.readFileSync(owner, "utf-8").trim() === "add") {
		this.addDependency(path.join(dirname, "overlap-dependency.txt"));
	}
	return source;
};
