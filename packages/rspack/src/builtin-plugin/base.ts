import * as binding from "@rspack/binding";
import { Compiler } from "..";

// TODO: workaround for https://github.com/napi-rs/napi-rs/pull/1690
export enum BuiltinPluginKind {
	Define = 0,
	Provide = 1,
	Banner = 2,
	Progress = 3,
	Copy = 4,
	Html = 5,
	SwcJsMinimizer = 6,
	SwcCssMinimizer = 7,
	Entry = 8
}

export abstract class RspackBuiltinPlugin {
	abstract raw(): binding.BuiltinPlugin;
	apply(compiler: Compiler) {
		compiler.__internal__registerBuiltinPlugin(this);
	}
}

export function create<T extends any[], R>(
	kind: BuiltinPluginKind,
	resolve: (...args: T) => R
) {
	return class Plugin extends RspackBuiltinPlugin {
		_options: R;

		constructor(...args: T) {
			super();
			this._options =
				resolve(...args) ??
				(false as R) /* undefined or null will cause napi error, so false for fallback */;
		}

		raw(): binding.BuiltinPlugin {
			return {
				kind: kind as any,
				options: this._options
			};
		}
	};
}
