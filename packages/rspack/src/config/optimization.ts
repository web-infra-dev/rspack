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
	sideEffects?: "flag" | boolean;
}

export interface ResolvedOptimization {
	moduleIds: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
	splitChunks?: RawSplitChunksOptions;
	removeAvailableModules?: boolean;
	sideEffects?: "flag" | "true" | "false";
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
		removeAvailableModules: op.removeAvailableModules ?? mode === "production",
		sideEffects: resolveSideEffects(op.sideEffects, mode)
	};
}

function resolveSideEffects(
	sideEffects: "flag" | boolean | undefined,
	mode: string
): "flag" | "true" | "false" {
	if (sideEffects === undefined) {
		return String(mode === "production") as "true" | "false";
	}
	if (typeof sideEffects === "boolean") {
		return sideEffects.toString() as "true" | "false";
	} else if (sideEffects === "flag") {
		return sideEffects;
	} else {
		return "false";
	}
}
