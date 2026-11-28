/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	this.emitWarning(new Error(source.trim()));
	return "";
};
