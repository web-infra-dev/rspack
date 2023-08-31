import * as binding from "@rspack/binding";
import { Compiler } from "..";

// TODO: workaround for https://github.com/napi-rs/napi-rs/pull/1690
export enum BuiltinPluginKind {
	Define = "Define",
	Provide = "Provide",
	Banner = "Banner",
	Progress = "Progress",
	Copy = "Copy",
	Html = "Html",
	SwcJsMinimizer = "SwcJsMinimizer",
	SwcCssMinimizer = "SwcCssMinimizer",
	Entry = "Entry"
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
