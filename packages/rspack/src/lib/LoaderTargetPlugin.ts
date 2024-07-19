/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/

import type { Compilation } from "../Compilation";
import type { Compiler } from "../Compiler";
import { NormalModule } from "../NormalModule";
import type { Target } from "../config";

class LoaderTargetPlugin {
	target: Target;

	constructor(target: Target) {
		this.target = target;
	}

	/**
	 * Apply the plugin
	 * @param {Compiler} compiler the compiler instance
	 * @returns {void}
	 */
	apply(compiler: Compiler): void {
		compiler.hooks.compilation.tap(
			"LoaderTargetPlugin",
			(compilation: Compilation) => {
				NormalModule.getCompilationHooks(compilation).loader.tap(
					"LoaderTargetPlugin",
					loaderContext => {
						loaderContext.target = this.target as Target;
					}
				);
			}
		);
	}
}

export { LoaderTargetPlugin };
