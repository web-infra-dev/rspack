import { z } from "zod";

export function snapshot() {
	return z.strictObject({
		module: z
			.strictObject({
				hash: z.boolean().optional(),
				timestamp: z.boolean().optional()
			})
			.optional(),
		resolve: z
			.strictObject({
				hash: z.boolean().optional(),
				timestamp: z.boolean().optional()
			})
			.optional()
	});
}
