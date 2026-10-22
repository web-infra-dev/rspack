/** @type {import("@rspack/core").LoaderDefinitionFunction} */
export default async function (source) {
	const classes = await this.importModule("./style.module.css");
	return `const classes = ${JSON.stringify(classes)};\n${source}`;
};
