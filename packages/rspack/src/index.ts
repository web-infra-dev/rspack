export * from "./compiler";
export * from "./multiCompiler";
export * from "./compilation";
export * from "./config";
export * from "./rspack";
export * from "./stats";
export * from "./multiStats";
export * from "./chunk_group";
export * from "./normalModuleFactory";

// TODO(hyf0): should remove this re-export when we cleanup the exports of `@rspack/core`
import { Configuration } from "./config";

export type OptimizationSplitChunksOptions = NonNullable<
	Configuration["optimization"]
>["splitChunks"];
