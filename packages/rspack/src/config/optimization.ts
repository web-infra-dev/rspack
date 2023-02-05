import { PluginInstance } from "./plugin";
import {
	OptimizationSplitChunksOptions,
	resolveSplitChunksOptions
} from "./splitChunks";
import type { RawSplitChunksOptions } from "@rspack/binding";

/**
 * Create an additional chunk which contains only the webpack runtime and chunk hash maps.
 */
export type OptimizationRuntimeChunk =
	| ("single" | "multiple")
	| boolean
	| {
			/**
			 * The name or name factory for the runtime chunks.
			 */
			name?: string | Function;
	  };
export type OptimizationRuntimeChunkNormalized =
	| false
	| {
			/**
			 * The name factory for the runtime chunks.
			 */
			name?: Function;
	  };

export interface Optimization {
	moduleIds?: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
	splitChunks?: OptimizationSplitChunksOptions;
	runtimeChunk?: OptimizationRuntimeChunk;
	removeAvailableModules?: boolean;
}

export interface ResolvedOptimization {
	moduleIds: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
	splitChunks?: RawSplitChunksOptions;
	removeAvailableModules?: boolean;
}

export function getNormalizedOptimizationRuntimeChunk(
	runtimeChunk: OptimizationRuntimeChunk
): OptimizationRuntimeChunkNormalized {
	// @ts-expect-error
	if (runtimeChunk === undefined) return undefined;
	if (runtimeChunk === false) return false;
	if (runtimeChunk === "single") {
		return {
			name: () => "runtime"
		};
	}
	if (runtimeChunk === true || runtimeChunk === "multiple") {
		return {
			// @ts-expect-error
			name: entrypoint => `runtime~${entrypoint.name}`
		};
	}
	const { name } = runtimeChunk;
	return {
		name: typeof name === "function" ? name : () => name
	};
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
		splitChunks: resolveSplitChunksOptions(op.splitChunks),
		removeAvailableModules: op.removeAvailableModules ?? mode === "production"
	};
}
