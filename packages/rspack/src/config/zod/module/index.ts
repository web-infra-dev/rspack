import { z } from "zod";
import { rules } from "./rules";
import { generator } from "./generator";

export function module() {
	return z.strictObject({
		defaultRules: rules().optional(),
		rules: rules().optional(),
		generator: generator().optional(),
		parser: z
			.strictObject({
				asset: z
					.strictObject({
						dataUrlCondition: z
							.strictObject({
								maxSize: z.number().optional()
							})
							.optional()
					})
					.optional()
			})
			.optional()
	});
}
