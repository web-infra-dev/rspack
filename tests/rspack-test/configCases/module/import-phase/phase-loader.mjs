/** @typedef {import("@rspack/core").LoaderDefinition<{ phase: string }>} LoaderDefinition */

/** @type {LoaderDefinition} */
export default function (source) {
	const options = this.getOptions();
	return `${source}\nexport default ${JSON.stringify(
		options.phase
	)};\n`;
};
