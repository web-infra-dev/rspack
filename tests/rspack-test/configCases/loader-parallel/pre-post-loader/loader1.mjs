/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	return source + 'module.exports += " loader1";\n';
};
