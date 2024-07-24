/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function (source) {
	this.emitWarning(new Error(source.trim()));
	return "";
};
