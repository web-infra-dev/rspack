import type { Compiler, RspackPluginInstance } from "../";
import type { IgnoreWarningPattern } from "../";

export default class IgnoreWarningsPlugin implements RspackPluginInstance {
	_ignorePattern: IgnoreWarningPattern;
	name = "IgnoreWarningsPlugin";

	constructor(ignorePattern: IgnoreWarningPattern) {
		this._ignorePattern = ignorePattern;
	}

	apply(compiler: Compiler) {
		compiler.hooks.compilation.tap("IgnoreWarningsPlugin", compilation => {
			//todo what compilation hook to use
			// filter the warning based on the pattern
			// compilation.hooks.
			console.log(`ignore based on ${this._ignorePattern}`);
		});
	}
}
