/** @type {import("@rspack/core").LoaderDefinition}} */
module.exports = function () {
	return `module.exports = ${JSON.stringify(!!this.hot)};`;
};
