import schema from "./loader-1.options.json" with { type: "json" };

/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	const options = this.getOptions(schema);

	const json = JSON.stringify(options)
		.replace(/\u2028/g, "\\u2028")
		.replace(/\u2029/g, "\\u2029");

	return `module.exports = ${json}`;
};
