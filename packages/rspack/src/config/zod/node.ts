import { z } from "zod";

export function node() {
	return z.literal(false).or(
		z
			.object({
				__dirname: z
					.boolean()
					.or(z.enum(["warn-mock", "mock", "eval-only"]))
					.optional(),
				__filename: z
					.boolean()
					.or(z.enum(["warn-mock", "mock", "eval-only"]))
					.optional(),
				global: z
					.boolean()
					.or(z.enum(["warn"]))
					.optional()
			})
			.strict()
	);
}
