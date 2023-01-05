import { PluginInstance } from "./plugin";
import { OptimizationSplitChunksOptions, resolveSplitChunksOptions } from "./splitChunks";
import type { RawSplitChunksOptions } from '@rspack/binding'

export interface Optimization {
	moduleIds?: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
	splitChunks?: OptimizationSplitChunksOptions,
}

export interface ResolvedOptimization {
	moduleIds: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
	splitChunks?: RawSplitChunksOptions,
}

export function resolveOptimizationOptions(
	op: Optimization,
	mode: string
): ResolvedOptimization {
	return {
		moduleIds:
			op.moduleIds ?? (mode === "production" ? "deterministic" : "named"),
		minimize: op.minimize,
		minimizer: op.minimizer,
		splitChunks: resolveSplitChunksOptions(op.splitChunks)
	};
}
