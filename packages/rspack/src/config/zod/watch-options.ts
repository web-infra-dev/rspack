import { z } from "zod";

export function watchOptions() {
	return z
		.object({
			aggregateTimeout: z.number().optional(),
			followSymlinks: z.boolean().optional(),
			ignored: z
				.string()
				.array()
				.or(z.instanceof(RegExp))
				.or(z.string())
				.optional(),
			poll: z.number().or(z.boolean()).optional(),
			stdin: z.boolean().optional()
		})
		.strict();
}
