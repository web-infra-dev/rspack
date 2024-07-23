/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/LoaderOptionsPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
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
