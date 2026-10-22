/** @type {import("@rspack/core").LoaderDefinition} */
export default function (src) {
	return `module.exports = "loader-a" + module.id`;
};
