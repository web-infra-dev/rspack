import fs from "node:fs";
import path from "node:path";

let runs = 0;

export default function () {
	const dependency = path.join(path.dirname(this.resourcePath), "overlap-dependency.txt");
	this.addDependency(dependency);
	runs++;
	return `module.exports = ${JSON.stringify({
		value: fs.readFileSync(dependency, "utf-8").trim(),
		runs
	})};`;
};
