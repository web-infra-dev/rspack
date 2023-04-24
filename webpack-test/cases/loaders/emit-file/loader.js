/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function (content) {
	this.emitFile("extra-file.js", content);
	return "";
};
