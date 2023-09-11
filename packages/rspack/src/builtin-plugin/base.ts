import * as binding from "@rspack/binding";
import { Compiler, RspackPluginInstance } from "..";

// TODO: workaround for https://github.com/napi-rs/napi-rs/pull/1690
export enum BuiltinPluginName {
	DefinePlugin = "DefinePlugin",
	ProvidePlugin = "ProvidePlugin",
	BannerPlugin = "BannerPlugin",
	ProgressPlugin = "ProgressPlugin",
	EntryPlugin = "EntryPlugin",
	ExternalsPlugin = "ExternalsPlugin",
	NodeTargetPlugin = "NodeTargetPlugin",
	ElectronTargetPlugin = "ElectronTargetPlugin",
	EnableChunkLoadingPlugin = "EnableChunkLoadingPlugin",
	CommonJsChunkFormatPlugin = "CommonJsChunkFormatPlugin",
	ArrayPushCallbackChunkFormatPlugin = "ArrayPushCallbackChunkFormatPlugin",
	ModuleChunkFormatPlugin = "ModuleChunkFormatPlugin",
	HttpExternalsRspackPlugin = "HttpExternalsRspackPlugin",
	CopyRspackPlugin = "CopyRspackPlugin",
	HtmlRspackPlugin = "HtmlRspackPlugin",
	SwcJsMinimizerRspackPlugin = "SwcJsMinimizerRspackPlugin",
	SwcCssMinimizerRspackPlugin = "SwcCssMinimizerRspackPlugin"
}

export abstract class RspackBuiltinPlugin implements RspackPluginInstance {
	abstract raw(): binding.BuiltinPlugin;
	abstract name: BuiltinPluginName;
	apply(compiler: Compiler) {
		compiler.__internal__registerBuiltinPlugin(this);
	}
}

export function create<T extends any[], R>(
	name: BuiltinPluginName,
	resolve: (...args: T) => R
) {
	return class Plugin extends RspackBuiltinPlugin {
		name = name;
		_options: R;

		constructor(...args: T) {
			super();
			this._options =
				resolve(...args) ??
				(false as R) /* undefined or null will cause napi error, so false for fallback */;
		}

		raw(): binding.BuiltinPlugin {
			return {
				name: name as any,
				options: this._options
			};
		}
	};
}
