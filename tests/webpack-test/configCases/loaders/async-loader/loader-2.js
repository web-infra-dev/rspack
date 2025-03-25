/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function () {
	return Promise.resolve(`module.exports = 'b';`);
};
