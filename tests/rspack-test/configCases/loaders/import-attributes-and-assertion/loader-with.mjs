/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	return JSON.stringify({ type: "with" });
};
