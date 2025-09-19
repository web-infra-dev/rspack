/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function (content) {
	return content.split("").reverse().join("");
};
