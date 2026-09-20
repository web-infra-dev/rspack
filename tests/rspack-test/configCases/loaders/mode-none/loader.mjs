/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	return `module.exports = "${this.mode}";`;
};
