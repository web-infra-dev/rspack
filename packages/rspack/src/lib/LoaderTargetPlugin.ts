/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/LoaderTargetPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

const NormalModule = require("../NormalModule");

// /** @typedef {import("./Compiler")} Compiler */
/** @typedef {any} Compiler */

class LoaderTargetPlugin {
	/**
	 * @param {string} target the target
	 */
	constructor(target) {
		this.target = target;
	}

	/**
	 * Apply the plugin
	 * @param {Compiler} compiler the compiler instance
	 * @returns {void}
	 */
	apply(compiler) {
		// @ts-expect-error
		compiler.hooks.compilation.tap("LoaderTargetPlugin", compilation => {
			// @ts-expect-error
			NormalModule.getCompilationHooks(compilation).loader.tap(
				"LoaderTargetPlugin",
				// @ts-expect-error
				loaderContext => {
					loaderContext.target = this.target;
				}
			);
		});
	}
}

export { LoaderTargetPlugin };
