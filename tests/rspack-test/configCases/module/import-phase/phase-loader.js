"use strict";

/** @typedef {import("@rspack/core").LoaderDefinition<{ phase: string }>} LoaderDefinition */

/** @type {LoaderDefinition} */
module.exports = function (source) {
	const options = this.getOptions();
	return `${source}\nexport default ${JSON.stringify(
		options.phase
	)};\n`;
};
