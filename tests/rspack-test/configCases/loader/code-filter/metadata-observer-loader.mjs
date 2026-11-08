import path from "node:path";

export default function (source, sourceMap, additionalData) {
	const filtered = this.loaders[this.loaderIndex + 1];
	if (!filtered?.loaderItem.data.pitched) {
		throw new Error("The filtered loader pitch function did not run");
	}
	if (sourceMap?.version !== 3 || additionalData?.marker !== "preserved") {
		throw new Error("The code filter changed source map or additional data");
	}
	const dependency = path.resolve(import.meta.dirname, "dependency.txt");
	if (!this.getDependencies().includes(dependency)) {
		throw new Error("The code filter changed loader dependencies");
	}
	this.callback(null, source, sourceMap, additionalData);
}
