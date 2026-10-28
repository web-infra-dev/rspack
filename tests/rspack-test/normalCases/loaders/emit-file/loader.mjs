/** @type {import("@rspack/core").LoaderDefinition} */
export default function (content) {
	this.emitFile("extra-file.js", content);
	return "";
};
