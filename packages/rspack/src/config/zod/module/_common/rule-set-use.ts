import { z } from "zod";

const ruleSetItem = z.string().or(
	z.strictObject({
		ident: z.string().optional(),
		loader: z.string().optional(),
		options: z.string().or(z.record(z.any())).optional()
	})
);

export function ruleSetUse() {
	return ruleSetItem.or(ruleSetItem.array());
}
