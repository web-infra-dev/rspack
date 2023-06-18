import { z } from "zod";

function inlineAssetGenerator() {
	return z.strictObject({
		dataUrl: z
			.strictObject({
				encoding: z.literal(false).or(z.literal("base64")).optional(),
				mimetype: z.string().optional()
			})
			.optional()
	});
}

function resourceAssetGenerator() {
	return z.strictObject({
		filename: z.string().optional(),
		publicPath: z.string().optional()
	});
}

export function generator() {
	return z.strictObject({
		"asset/inline": inlineAssetGenerator().optional(),
		"asset/resource": resourceAssetGenerator().optional(),
		asset: inlineAssetGenerator().merge(resourceAssetGenerator()).optional()
	});
}
