import { string, z } from "zod";

function chunks() {
	return z.enum(["initial", "async", "all"]).or(z.instanceof(RegExp));
}

function name() {
	return z.string().or(z.literal(false));
}

const sharedCacheGroupConfigPart = {
	chunks: chunks().optional(),
	minChunks: z.number().optional(),
	name: name().optional(),
	minSize: z.number().optional(),
	maxSize: z.number().optional(),
	maxAsyncSize: z.number().optional(),
	maxInitialSize: z.number().optional()
};

const cacheGroupOptions = z.strictObject({
	test: z.instanceof(RegExp).or(z.string()).optional(),
	priority: z.number().optional(),
	enforce: z.boolean().optional(),
	reuseExistingChunk: z.boolean().optional(),
	type: z.string().or(z.instanceof(RegExp)).optional(),
	idHint: z.string().optional(),
	...sharedCacheGroupConfigPart
});

export function splitChunks() {
	return z.literal(false).or(
		// We use loose object here to prevent breaking change on config
		z.object({
			cacheGroups: z.record(z.literal(false).or(cacheGroupOptions)).optional(),
			maxAsyncRequests: z.number().optional(),
			maxInitialRequests: z.number().optional(),
			fallbackCacheGroup: z
				.strictObject({
					chunks: chunks().optional(),
					minSize: z.number().optional(),
					maxSize: z.number().optional(),
					maxAsyncSize: z.number().optional(),
					maxInitialSize: z.number().optional()
				})
				.optional(),
			...sharedCacheGroupConfigPart
		})
	);
}

export type SplitChunksConfig = z.TypeOf<ReturnType<typeof splitChunks>>;
export type CacheGroupOptionsConfig = z.TypeOf<typeof cacheGroupOptions>;
