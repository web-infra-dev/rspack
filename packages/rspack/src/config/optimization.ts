import { PluginInstance } from "./plugin";

export interface Optimization {
	moduleIds?: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
}

export interface ResolvedOptimization {
	moduleIds?: "named" | "deterministic" | "natural";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
	removeAvailableModules?: boolean;
	removeEmptyChunks?: boolean;
	mergeDuplicateChunks?: boolean;
	flagIncludedChunks?: boolean;
	chunkIds?: "deterministic" | "natural" | "named";
	sideEffects?: true | "flag";
	providedExports?: boolean;
	usedExports?: boolean;
	innerGraph?: boolean;
	mangleExports?: boolean;
	concatenateModules?: boolean;
	runtimeChunk?: boolean;
	emitOnErrors?: boolean;
	checkWasmTypes?: boolean;
	mangleWasmImports?: boolean;
	portableRecords?: boolean;
	realContentHash?: boolean;
	nodeEnv?: false | "production" | "development";
}
export function resolveOptimizationOptions(
	op: Optimization,
	mode: string
): ResolvedOptimization {
	return {
		moduleIds:
			op.moduleIds ?? mode === "production" ? "deterministic" : "named",
		minimize: op.minimize,
		minimizer: op.minimizer
	};
}
