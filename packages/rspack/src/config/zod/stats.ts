import { z } from "zod";

export function stats() {
	return z
		.enum(["none", "errors-only", "errors-warnings", "normal", "verbose"])
		.or(z.boolean())
		.or(
			z.object({
				all: z.boolean().optional(),
				assets: z.boolean().optional(),
				chunkGroups: z.boolean().optional(),
				chunks: z.boolean().optional(),
				colors: z.boolean().optional(),
				entrypoints: z.boolean().optional(),
				errors: z.boolean().optional(),
				errorsCount: z.boolean().optional(),
				hash: z.boolean().optional(),
				modules: z.boolean().optional(),
				preset: z
					.enum(["normal", "none", "verbose", "errors-only", "errors-warnings"])
					.optional(),
				publicPath: z.boolean().optional(),
				reasons: z.boolean().optional(),
				warnings: z.boolean().optional(),
				warningsCount: z.boolean().optional(),
				outputPath: z.boolean().optional(),
				chunkModules: z.boolean().optional(),
				chunkRelations: z.boolean().optional(),
				timings: z.boolean().optional(),
				builtAt: z.boolean().optional(),
				nestedModules: z.boolean().optional(),
				source: z.boolean().optional()
			})
		);
}
