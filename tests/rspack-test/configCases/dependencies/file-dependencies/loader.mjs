import path from "node:path";

/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	this.addDependency(path.resolve(import.meta.dirname, "node_modules/package/extra.js"));
	this.addDependency(path.resolve(import.meta.dirname, "extra.js"));
	return source;
};
