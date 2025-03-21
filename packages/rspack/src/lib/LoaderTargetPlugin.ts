/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/LoaderTargetPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Compiler } from "../Compiler";
import { NormalModule } from "../NormalModule";
import type { Target } from "../config";

export class LoaderTargetPlugin {
	/**
	 * @param target the target
	 */
	constructor(public readonly target: Target) {}

	/**
	 * Apply the plugin
	 * @param compiler the compiler instance
	 * @returns
	 */
	apply(compiler: Compiler): void {
		compiler.hooks.compilation.tap("LoaderTargetPlugin", compilation => {
			NormalModule.getCompilationHooks(compilation).loader.tap(
				"LoaderTargetPlugin",
				loaderContext => {
					loaderContext.target = this.target;
				}
			);
		});
	}
}
