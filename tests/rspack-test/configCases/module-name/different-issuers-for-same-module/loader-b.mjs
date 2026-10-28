/** @type {import("@rspack/core").LoaderDefinition} */
export default function (src) {
	return `module.exports = "loader-b" + module.id`;
};
