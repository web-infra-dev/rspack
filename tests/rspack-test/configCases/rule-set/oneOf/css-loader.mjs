/** @type {import("@rspack/core").LoaderDefinition<{ get(): string }>} */
export default function (source) {
	return "module.exports='__css__'";
};
