import path from "node:path";

export default function (source) {
	this.addDependency(path.join(path.dirname(this.resourcePath), "chain-right.txt"));
	return source;
};
