import path from "node:path";

export default function (content) {
	this.addContextDependency(path.resolve(import.meta.dirname, "./missing-directory"));
	return content;
}
