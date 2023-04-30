import type {
	Compiler,
	IgnoreWarningsNormalized,
	RspackPluginInstance
} from "../";

export default class IgnoreWarningsPlugin implements RspackPluginInstance {
	_ignorePattern: IgnoreWarningsNormalized;
	name = "IgnoreWarningsPlugin";

	constructor(ignorePattern: IgnoreWarningsNormalized) {
		this._ignorePattern = ignorePattern;
	}

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
