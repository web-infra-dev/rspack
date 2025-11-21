/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function (source) {
	return "module.exports = " + JSON.stringify("loader matched");
};
