/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function (source) {
	return JSON.stringify({ type: "with" });
};
