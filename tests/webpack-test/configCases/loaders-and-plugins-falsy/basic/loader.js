/** @type {import("@rspack/core").LoaderDefinition<{ value: any }>} */
module.exports = function loader(content) {
	return content.replace(/test/, "NEW");
};
