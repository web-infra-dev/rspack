/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/

"use strict";

const ModuleFilenameHelpers = require("./ModuleFilenameHelpers");
const { NormalModule } = require("../NormalModule");

// /** @typedef {import("../declarations/plugins/LoaderOptionsPlugin").LoaderOptionsPluginOptions} LoaderOptionsPluginOptions */
/** @typedef {import("../Compiler").Compiler} Compiler */
/** @typedef {any} LoaderOptionsPluginOptions */

class LoaderOptionsPlugin {
	/**
	 * @param {LoaderOptionsPluginOptions} options options object
	 */
	constructor(options = {}) {
		if (typeof options !== "object") options = {};
		if (!options.test) {
			options.test = {
				test: () => true
			};
		}
		this.options = options;
	}

	/**
	 * Apply the plugin
	 * @param {Compiler} compiler the compiler instance
	 * @returns {void}
	 */
	apply(compiler) {
		const options = this.options;
		compiler.hooks.compilation.tap("LoaderOptionsPlugin", compilation => {
			NormalModule.getCompilationHooks(compilation).loader.tap(
				"LoaderOptionsPlugin",
				context => {
					const resource = context.resourcePath;
					if (!resource) return;
					if (ModuleFilenameHelpers.matchObject(options, resource)) {
						for (const key of Object.keys(options)) {
							if (key === "include" || key === "exclude" || key === "test") {
								continue;
							}
							// @ts-expect-error
							context[key] = options[key];
						}
					}
				}
			);
		});
	}
}

export { LoaderOptionsPlugin };
