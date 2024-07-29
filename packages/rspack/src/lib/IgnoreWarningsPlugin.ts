/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/IgnoreWarningsPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type {
	Compiler,
	IgnoreWarningsNormalized,
	RspackPluginInstance
} from "..";

class IgnoreWarningsPlugin implements RspackPluginInstance {
	_ignorePattern: IgnoreWarningsNormalized;
	name = "IgnoreWarningsPlugin";

	/**
	 * @param ignoreWarnings conditions to ignore warnings
	 */
	constructor(ignorePattern: IgnoreWarningsNormalized) {
		this._ignorePattern = ignorePattern;
	}

	/**
	 * Apply the plugin
	 * @param compiler the compiler instance
	 * @returns
	 */
	apply(compiler: Compiler) {
		compiler.hooks.compilation.tap(this.name, compilation => {
			compilation.hooks.processWarnings.tap(this.name, warnings => {
				return warnings.filter(warning => {
					return !this._ignorePattern.some(ignore =>
						ignore(warning, compilation)
					);
				});
			});
		});
	}
}

export default IgnoreWarningsPlugin;
