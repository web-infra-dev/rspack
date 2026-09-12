/** @type {import("@rspack/core").LoaderDefinition<{ tag: string }>} */
module.exports = function (source) {
	const { tag } = this.getOptions();
	return `${source}\nmodule.exports += ${JSON.stringify(tag)};`;
};
