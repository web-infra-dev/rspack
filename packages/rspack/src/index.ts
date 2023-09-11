import type Watching from "./Watching";
export * from "./Compiler";
export * from "./MultiCompiler";
export * from "./Compilation";
export * from "./config";
export * from "./rspack";
export { RuntimeGlobals } from "./RuntimeGlobals";
export * from "./Stats";
export * from "./MultiStats";
export * from "./ChunkGroup";
export * from "./NormalModuleFactory";
export { cachedCleverMerge as cleverMerge } from "./util/cleverMerge";
export { EnvironmentPlugin } from "./lib/EnvironmentPlugin";
export { LoaderOptionsPlugin } from "./lib/LoaderOptionsPlugin";
export {
	registerGlobalTrace as experimental_registerGlobalTrace,
	cleanupGlobalTrace as experimental_cleanupGlobalTrace
} from "@rspack/binding";
import { Configuration } from "./config";
// TODO(hyf0): should remove this re-export when we cleanup the exports of `@rspack/core`
export type OptimizationSplitChunksOptions = NonNullable<
	Configuration["optimization"]
>["splitChunks"];
export {
	BannerPlugin,
	DefinePlugin,
	ProvidePlugin,
	ProgressPlugin,
	HtmlRspackPlugin,
	SwcJsMinimizerRspackPlugin,
	SwcCssMinimizerRspackPlugin,
	CopyRspackPlugin,
	EntryPlugin,
	ExternalsPlugin,
	EnableChunkLoadingPlugin
} from "./builtin-plugin";
export type {
	BannerPluginArgument,
	DefinePluginOptions,
	ProvidePluginOptions,
	ProgressPluginArgument,
	HtmlRspackPluginOptions,
	SwcJsMinimizerRspackPluginOptions,
	CopyRspackPluginOptions,
	EntryOptions
} from "./builtin-plugin";
import { ElectronTargetPlugin, NodeTargetPlugin } from "./builtin-plugin";
import NodeTemplatePlugin from "./node/NodeTemplatePlugin";
export const node = { NodeTargetPlugin, NodeTemplatePlugin };
export const electron = { ElectronTargetPlugin };

export { Watching };
