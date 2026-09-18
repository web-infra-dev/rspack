/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	return source + 'module.exports += " loader2";\n';
};
