export * from "./APIPlugin";
export * from "./ArrayPushCallbackChunkFormatPlugin";
export * from "./AssetModulesPlugin";
export * from "./AsyncWebAssemblyModulesPlugin";
export * from "./BannerPlugin";
export { RspackBuiltinPlugin } from "./base";
export * from "./BundlerInfoRspackPlugin";
export * from "./ChunkPrefetchPreloadPlugin";
export * from "./CommonJsChunkFormatPlugin";
export * from "./CopyRspackPlugin";
export * from "./css-extract";
export * from "./CssModulesPlugin";
export * from "./DataUriPlugin";
export * from "./DefinePlugin";
export * from "./DeterministicChunkIdsPlugin";
export * from "./DeterministicModuleIdsPlugin";
export * from "./DynamicEntryPlugin";
export * from "./ElectronTargetPlugin";
export * from "./EnableChunkLoadingPlugin";
export * from "./EnableLibraryPlugin";
export * from "./EnableWasmLoadingPlugin";
export * from "./EnsureChunkConditionsPlugin";
export * from "./EntryPlugin";
export * from "./EvalDevToolModulePlugin";
export * from "./EvalSourceMapDevToolPlugin";
export * from "./ExternalsPlugin";
export * from "./FileUriPlugin";
export * from "./FlagDependencyExportsPlugin";
export * from "./FlagDependencyUsagePlugin";
export * from "./HotModuleReplacementPlugin";
export * from "./HtmlRspackPlugin";
export * from "./HttpExternalsRspackPlugin";
export * from "./IgnorePlugin";
export * from "./InferAsyncModulesPlugin";
export * from "./JavascriptModulesPlugin";
export * from "./JsLoaderRspackPlugin";
export * from "./JsonModulesPlugin";
export * from "./lazy-compilation/plugin";
export * from "./LimitChunkCountPlugin";
export * from "./MangleExportsPlugin";
export * from "./MergeDuplicateChunksPlugin";
export * from "./ModuleChunkFormatPlugin";
export * from "./ModuleConcatenationPlugin";
export * from "./NamedChunkIdsPlugin";
export * from "./NamedModuleIdsPlugin";
export * from "./NaturalChunkIdsPlugin";
export * from "./NaturalModuleIdsPlugin";
export * from "./NodeTargetPlugin";
export * from "./ProgressPlugin";
export * from "./ProvidePlugin";
export * from "./RealContentHashPlugin";
export * from "./RemoveEmptyChunksPlugin";
export * from "./RuntimeChunkPlugin";
export * from "./RuntimePlugin";
export * from "./SideEffectsFlagPlugin";
export * from "./SizeLimitsPlugin";
export * from "./SourceMapDevToolPlugin";
export * from "./SplitChunksPlugin";
export * from "./SwcCssMinimizerPlugin";
export * from "./LightningCssMiminizerRspackPlugin";
export * from "./SwcJsMinimizerPlugin";
export * from "./WarnCaseSensitiveModulesPlugin";
export * from "./WebWorkerTemplatePlugin";
export * from "./WorkerPlugin";

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
