import { z } from "zod";

export function infrastructureLogging() {
	return z
		.object({
			appendOnly: z.boolean().optional(),
			colors: z.boolean().optional(),
			console: z.boolean().optional(),
			debug: z.boolean().or(z.any()).optional(),
			level: z
				.enum(["none", "error", "warn", "info", "log", "verbose"])
				.optional(),
			stream: z.boolean().optional()
		})
		.strict();
}
