import * as binding from "@rspack/binding";

// TODO: workaround for https://github.com/napi-rs/napi-rs/pull/1690
export enum BuiltinPluginKind {
	Define = 0,
	Provide = 1,
	Banner = 2,
	SwcJsMinimizer = 3,
	SwcCssMinimizer = 4,
	PresetEnv = 5,
	TreeShaking = 6,
	ReactOptions = 7,
	DecoratorOptions = 8,
	NoEmitAssets = 9,
	Emotion = 10,
	Relay = 11,
	PluginImport = 12,
	DevFriendlySplitChunks = 13,
	Progress = 14,
	Copy = 15,
	Html = 16
}

export abstract class BuiltinPlugin {
	abstract raw(): binding.BuiltinPlugin;
}

export function create<T, R>(
	kind: BuiltinPluginKind,
	resolve: (options: T) => R
) {
	return class Plugin extends BuiltinPlugin {
		#options: R;

		constructor(options: T) {
			super();
			this.#options = resolve(options);
		}

		raw(): binding.BuiltinPlugin {
			return {
				kind,
				options: this.#options
			};
		}
	};
}
