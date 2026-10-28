/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	this.emitWarning(new Error("this is a warning"));
	this.emitError(new Error("this is an error"));
	return source;
};
