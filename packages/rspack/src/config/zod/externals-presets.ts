import { z } from "zod";

export function externalsPresets() {
	return z
		.object({
			web: z.boolean().optional(),
			node: z.boolean().optional(),
			electron: z.boolean().optional(),
			electronMain: z.boolean().optional(),
			electronPreload: z.boolean().optional(),
			electronRenderer: z.boolean().optional()
		})
		.strict();
}
