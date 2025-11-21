/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function (source) {
	return `module.exports = "${this.mode}";`;
};
