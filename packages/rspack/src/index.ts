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
export { EnvironmentPlugin } from "./lib/EnvironmentPlugin";
export { BannerPlugin } from "./lib/BannerPlugin";
import { Configuration } from "./config";
// TODO(hyf0): should remove this re-export when we cleanup the exports of `@rspack/core`
export type OptimizationSplitChunksOptions = NonNullable<
	Configuration["optimization"]
>["splitChunks"];
