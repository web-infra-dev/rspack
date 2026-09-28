/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	return Promise.resolve(`module.exports = 'b';`);
};
