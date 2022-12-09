export interface Optimization {
	moduleIds?: "named" | "deterministic";
}

export interface ResolvedOptimization {
	moduleIds: "named" | "deterministic";
}

export function resolveOptimizationOptions(
	op: Optimization,
	mode: string
): ResolvedOptimization {
	return {
		moduleIds: op.moduleIds ?? mode === "production" ? "deterministic" : "named"
	};
}
