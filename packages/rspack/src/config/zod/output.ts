import { z } from "zod";

function chunkLoadingType() {
	return z
		.enum(["jsonp", "import-scripts", "require", "async-node", "import"])
		.or(z.string());
}

function wasmLoadingType() {
	return z.enum(["...", "fetch-streaming", "fetch", "async-node"]);
}

export function publicPath() {
	return z.literal("auto").or(z.string());
}

function libraryType() {
	return z.enum([
		"...",
		"var",
		"module",
		"assign",
		"assign-properties",
		"this",
		"window",
		"self",
		"global",
		"commonjs",
		"commonjs2",
		"commonjs-module",
		"commonjs-static",
		"amd",
		"amd-require",
		"umd",
		"umd2",
		"jsonp",
		"system"
	]);
}

const umdNamedDefine = z.boolean();

const auxiliaryComment = z.string().or(
	z.strictObject({
		amd: z.string().optional(),
		commonjs: z.string().optional(),
		commonjs2: z.string().optional(),
		root: z.string().optional()
	})
);

const libraryName = z
	.string()
	.or(z.string().array())
	.or(
		z.strictObject({
			amd: z.string().optional(),
			commonjs: z.string().optional(),
			root: z.string().or(z.string().array()).optional()
		})
	);

const libraryOptions = z.strictObject({
	auxiliaryComment: auxiliaryComment.optional(),
	export: z.string().array().or(z.string()).optional(),
	name: libraryName.optional(),
	type: libraryType().optional(),
	umdNamedDefine: umdNamedDefine.optional()
});

export function output() {
	return z.strictObject({
		iife: z.boolean().optional(),
		clean: z.boolean().optional(),
		assetModuleFilename: z.string().optional(),
		auxiliaryComment: auxiliaryComment.optional(),
		chunkFormat: z
			.enum(["array-push", "commonjs", "module"])
			.or(z.literal(false))
			.optional(),
		chunkLoading: z.literal(false).or(chunkLoadingType()).optional(),
		enabledChunkLoadingTypes: chunkLoadingType().array().optional(),
		chunkFilename: z.string().optional(),
		cssChunkFilename: z.string().optional(),
		cssFilename: z.string().optional(),
		hotUpdateChunkFilename: z.string().optional(),
		hotUpdateMainFilename: z.string().optional(),
		webassemblyModuleFilename: z.string().optional(),
		hashSalt: z.string().optional(),
		filename: z.string().optional(),
		sourceMapFilename: z.string().optional(),
		importFunctionName: z.string().optional(),
		publicPath: publicPath().optional(),
		uniqueName: z.string().optional(),
		path: z.string().optional(),
		crossOriginLoading: z
			.literal(false)
			.or(z.enum(["anonymous", "use-credentials"]))
			.optional(),
		enabledWasmLoadingTypes: wasmLoadingType().array().optional(),
		wasmLoading: z.literal(false).or(wasmLoadingType()).optional(),
		enabledLibraryTypes: libraryType().or(libraryType().array()).optional(),
		globalObject: z.string().min(1).optional(),
		libraryExport: z.string().min(1).or(z.string().min(1).array()).optional(),
		libraryTarget: libraryType().optional(),
		hashFunction: z.string().or(z.function()).optional(),
		// TODO(hyf0)
		module: z.any().optional(),
		strictModuleErrorHandling: z.boolean().optional(),
		umdNamedDefine: umdNamedDefine.optional(),
		chunkLoadingGlobal: z.string().optional(),
		trustedTypes: z
			.literal(true)
			.or(z.string())
			.or(
				z.strictObject({
					policyName: z.string().optional()
				})
			)
			.optional(),
		hashDigest: z.string().optional(),
		hashDigestLength: z.number().optional(),
		library: libraryName.or(libraryOptions).optional(),
		asyncChunks: z.boolean().optional()
	});
}
