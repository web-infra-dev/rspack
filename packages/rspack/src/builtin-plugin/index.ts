export { RspackBuiltinPlugin } from "./base";

export * from "./DefinePlugin";
export * from "./ProvidePlugin";
export * from "./BannerPlugin";
export * from "./ProgressPlugin";
export * from "./EntryPlugin";
export * from "./ExternalsPlugin";
export * from "./NodeTargetPlugin";
export * from "./ElectronTargetPlugin";
export * from "./HttpExternalsRspackPlugin";
export * from "./EnableChunkLoadingPlugin";
export * from "./EnableLibraryPlugin";
export * from "./EnableWasmLoadingPlugin";
export * from "./ChunkPrefetchPreloadPlugin";
export * from "./ArrayPushCallbackChunkFormatPlugin";
export * from "./CommonJsChunkFormatPlugin";
export * from "./ModuleChunkFormatPlugin";
export * from "./HotModuleReplacementPlugin";
export * from "./WebWorkerTemplatePlugin";
export * from "./WorkerPlugin";
export * from "./LimitChunkCountPlugin";
export * from "./MergeDuplicateChunksPlugin";
export * from "./SplitChunksPlugin";
export * from "./NamedModuleIdsPlugin";
export * from "./DeterministicModuleIdsPlugin";
export * from "./NamedChunkIdsPlugin";
export * from "./DeterministicChunkIdsPlugin";
export * from "./RealContentHashPlugin";
export * from "./RemoveEmptyChunksPlugin";
export * from "./EnsureChunkConditionsPlugin";
export * from "./WarnCaseSensitiveModulesPlugin";
export * from "./DataUriPlugin";
export * from "./FileUriPlugin";
export * from "./RuntimePlugin";
export * from "./JsonModulesPlugin";
export * from "./InferAsyncModulesPlugin";
export * from "./JavascriptModulesPlugin";
export * from "./AsyncWebAssemblyModulesPlugin";
export * from "./AssetModulesPlugin";
export * from "./SourceMapDevToolPlugin";
export * from "./EvalSourceMapDevToolPlugin";
export * from "./EvalDevToolModulePlugin";
export * from "./SideEffectsFlagPlugin";
export * from "./FlagDependencyExportsPlugin";
export * from "./FlagDependencyUsagePlugin";
export * from "./MangleExportsPlugin";
export * from "./BundlerInfoRspackPlugin";
export * from "./ModuleConcatenationPlugin";
export * from "./CssModulesPlugin";

export * from "./HtmlRspackPlugin";
export * from "./CopyRspackPlugin";
export * from "./SwcJsMinimizerPlugin";
export * from "./SwcCssMinimizerPlugin";

export * from "./JsLoaderRspackPlugin";
export * from "./css-extract";

///// DEPRECATED /////
import { RawBuiltins } from "@rspack/binding";
import { RspackOptionsNormalized } from "..";

function resolveTreeShaking(
	treeShaking: Builtins["treeShaking"],
	production: boolean
): string {
	return treeShaking !== undefined
		? treeShaking.toString()
		: production
			? "true"
			: "false";
}

export interface Builtins {
	treeShaking?: boolean | "module";
}

export function deprecated_resolveBuiltins(
	builtins: Builtins,
	options: RspackOptionsNormalized
): RawBuiltins {
	const production = options.mode === "production" || !options.mode;

	return {
		treeShaking: resolveTreeShaking(builtins.treeShaking, production)
	};
}
