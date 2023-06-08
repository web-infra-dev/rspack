import { z } from "zod";

export function builtins() {
	// TODO(hyf0): need to use `strictObject` mode when developer have time to finish the whole schema
	return z.object({
		postcss: z
			.strictObject({
				pxtorem: z
					.strictObject({
						rootValue: z.number().optional(),
						unitPrecision: z.number().optional(),
						selectorBlackList: z.string().array().optional(),
						propList: z.string().array().optional(),
						replace: z.boolean().optional(),
						mediaQuery: z.boolean().optional(),
						minPixelValue: z.number().optional()
					})
					.optional()
			})
			.refine(() => {
				console.warn(
					"warn: `builtins.postcss` is going to be deprecated and will be removed at 0.3. See details at https://github.com/web-infra-dev/rspack/issues/3452"
				);
				return true;
			})
			.optional()
	});
}
