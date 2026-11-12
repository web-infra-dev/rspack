/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	if (source.indexOf("error") >= 0) throw new Error(source.trim());
	return source;
};
