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
	EnableLibraryPlugin = "EnableLibraryPlugin",
	EnableWasmLoadingPlugin = "EnableWasmLoadingPlugin",
	CommonJsChunkFormatPlugin = "CommonJsChunkFormatPlugin",
	ArrayPushCallbackChunkFormatPlugin = "ArrayPushCallbackChunkFormatPlugin",
	ModuleChunkFormatPlugin = "ModuleChunkFormatPlugin",
	HotModuleReplacementPlugin = "HotModuleReplacementPlugin",
	HttpExternalsRspackPlugin = "HttpExternalsRspackPlugin",
	CopyRspackPlugin = "CopyRspackPlugin",
	HtmlRspackPlugin = "HtmlRspackPlugin",
	SwcJsMinimizerRspackPlugin = "SwcJsMinimizerRspackPlugin",
	SwcCssMinimizerRspackPlugin = "SwcCssMinimizerRspackPlugin",
	LimitChunkCountPlugin = "LimitChunkCountPlugin",
	WebWorkerTemplatePlugin = "WebWorkerTemplatePlugin",
	MergeDuplicateChunksPlugin = "MergeDuplicateChunksPlugin",
	SplitChunksPlugin = "SplitChunksPlugin",
	OldSplitChunksPlugin = "OldSplitChunksPlugin",
	ContainerPlugin = "ContainerPlugin",
	ContainerReferencePlugin = "ContainerReferencePlugin",
	ModuleFederationRuntimePlugin = "ModuleFederationRuntimePlugin",
	ProvideSharedPlugin = "ProvideSharedPlugin",
	ConsumeSharedPlugin = "ConsumeSharedPlugin"
}

export abstract class RspackBuiltinPlugin implements RspackPluginInstance {
	abstract raw(compiler: Compiler): binding.BuiltinPlugin | null;
	abstract name: BuiltinPluginName;
	apply(compiler: Compiler) {
		let raw = this.raw(compiler);
		if (raw) compiler.__internal__registerBuiltinPlugin(raw);
	}
}

export function create<T extends any[], R>(
	name: BuiltinPluginName,
	resolve: (...args: T) => R
) {
	class Plugin extends RspackBuiltinPlugin {
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
	}

	// Make the plugin class name consistent with webpack
	// https://stackoverflow.com/a/46132163
	Object.defineProperty(Plugin, "name", { value: name });

	return Plugin;
}
