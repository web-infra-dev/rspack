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
					"warn: `builtins.postcss` is removed in 0.3.0. Please use `postcss-loader` instead."
				);
				return true;
			})
			.optional(),
		html: z
			.object({
				title: z.string().optional(),
				filename: z.string().optional(),
				template: z.string().optional(),
				templateParameters: z.record(z.string()).optional(),
				inject: z.enum(["head", "body"]).optional(),
				publicPath: z.string().optional(),
				scriptLoading: z.enum(["blocking", "defer", "module"]).optional(),
				chunks: z.string().array().optional(),
				excludedChunks: z.string().array().optional(),
				sri: z.enum(["sha256", "sha384", "sha512"]).optional(),
				minify: z.boolean().optional(),
				favicon: z.string().optional(),
				meta: z.record(z.string().or(z.record(z.string()))).optional()
			})
			.array()
			.optional()
	});
}
