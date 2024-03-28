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

export * from "./HtmlRspackPlugin";
export * from "./CopyRspackPlugin";
export * from "./SwcJsMinimizerPlugin";
export * from "./SwcCssMinimizerPlugin";

export * from "./JsLoaderRspackPlugin";
export * from "./lazy-compilation/plugin";

///// DEPRECATED /////
import { RawBuiltins, RawCssModulesConfig } from "@rspack/binding";
import { RspackOptionsNormalized } from "..";

type BuiltinsCssConfig = {
	modules?: Partial<RawCssModulesConfig>;
	namedExports?: boolean;
};

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
	css?: BuiltinsCssConfig;
	treeShaking?: boolean | "module";
}

export function deprecated_resolveBuiltins(
	builtins: Builtins,
	options: RspackOptionsNormalized
): RawBuiltins {
	const production = options.mode === "production" || !options.mode;

	return {
		// TODO: discuss with webpack, this should move to css generator options
		css: options.experiments.css
			? {
					modules: {
						localsConvention: "asIs",
						localIdentName: production
							? "[hash]"
							: "[path][name][ext]__[local]",
						exportsOnly: false,
						...builtins.css?.modules
					},
					namedExports: builtins.css?.namedExports
			  }
			: undefined,
		treeShaking: resolveTreeShaking(builtins.treeShaking, production)
	};
}
