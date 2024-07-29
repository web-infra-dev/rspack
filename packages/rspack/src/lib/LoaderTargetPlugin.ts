/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
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
