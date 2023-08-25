import { z } from "zod";

export function experiments() {
	return z.object({
		asyncWebAssembly: z.boolean().optional(),
		incrementalRebuild: z
			.boolean()
			.or(
				z.strictObject({
					make: z.boolean().optional(),
					emitAsset: z.boolean().optional()
				})
			)
			.optional(),
		lazyCompilation: z.boolean().optional(),
		outputModule: z.boolean().optional(),
		newSplitChunks: z.boolean().optional(),
		css: z.boolean().optional()
	});
}
