/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function loader(content) {
	return content + `.using-loader { color: red; }`;
};
