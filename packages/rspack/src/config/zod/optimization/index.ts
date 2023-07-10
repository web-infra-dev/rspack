import { z } from "zod";
import { Compiler } from "../../../Compiler";
import { splitChunks } from "./split-chunks";

const rspackPluginInstance = z.object({
	apply: z.function()
});

export function optimization() {
	return z.strictObject({
		moduleIds: z.enum(["named", "deterministic"]).optional(),
		minimize: z.boolean().optional(),
		minimizer: z.literal("...").or(rspackPluginInstance).array().optional(),
		splitChunks: splitChunks().optional(),
		runtimeChunk: z
			.enum(["single", "multiple"])
			.or(z.boolean())
			.or(
				z.strictObject({
					name: z
						.string()
						.or(z.function().returns(z.string().or(z.undefined())))
						.optional()
				})
			)
			.optional(),
		removeAvailableModules: z.boolean().optional(),
		removeEmptyChunks: z.boolean().optional(),
		realContentHash: z.boolean().optional(),
		sideEffects: z.enum(["flag"]).or(z.boolean()).optional()
	});
}

export type OptimizationConfig = z.TypeOf<ReturnType<typeof optimization>>;

export type OptimizationRuntimeChunkConfig = NonNullable<
	OptimizationConfig["runtimeChunk"]
>;
