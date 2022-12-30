import { PluginInstance } from "./plugin";

export interface Optimization {
	moduleIds?: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
}

export interface ResolvedOptimization {
	moduleIds: "named" | "deterministic";
	minimize?: boolean;
	minimizer?: ("..." | PluginInstance)[];
}
export function resolveOptimizationOptions(
	op: Optimization,
	mode: string
): ResolvedOptimization {
	return {
		moduleIds:
			op.moduleIds ?? (mode === "production" ? "deterministic" : "named"),
		minimize: op.minimize,
		minimizer: op.minimizer
	};
}
