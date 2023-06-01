import { z } from "zod";

export function externalsPresets() {
	return z
		.object({
			node: z.boolean()
		})
		.strict();
}
