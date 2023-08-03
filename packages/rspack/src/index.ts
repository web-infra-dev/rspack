export * from "./compiler";
export * from "./multiCompiler";
export * from "./compilation";
export * from "./config";
export * from "./rspack";
export * from "./stats";
export * from "./multiStats";
export * from "./chunk_group";
export * from "./normalModuleFactory";
export { cachedCleverMerge as cleverMerge } from "./util/cleverMerge";
export { BannerPlugin } from "./lib/BannerPlugin";
export { EnvironmentPlugin } from "./lib/EnvironmentPlugin";
export { LoaderOptionsPlugin } from "./lib/LoaderOptionsPlugin";
import { RspackOptions } from "./config";
// TODO(hyf0): should remove this re-export when we cleanup the exports of `@rspack/core`
export type OptimizationSplitChunksOptions = NonNullable<
	RspackOptions["optimization"]
>["splitChunks"];
