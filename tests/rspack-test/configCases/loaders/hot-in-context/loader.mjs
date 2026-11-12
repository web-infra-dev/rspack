/** @type {import("@rspack/core").LoaderDefinition}} */
export default function () {
	return `module.exports = ${JSON.stringify(!!this.hot)};`;
};
