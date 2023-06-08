import { z } from "zod";

export function experiments() {
	return z.object({
		asyncWebAssembly: z.boolean().optional(),
		incrementalRebuild: z.boolean().optional(),
		lazyCompilation: z.boolean().optional(),
		outputModule: z.boolean().optional(),
		newSplitChunks: z.boolean().optional(),
		css: z.boolean().optional()
	});
}
