/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	const callback = this.async();

	callback(null, `module.exports = 'c';`);
};
