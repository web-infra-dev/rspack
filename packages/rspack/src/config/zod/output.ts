import { z } from "zod";

function chunkLoadingType() {
	return z
		.enum(["jsonp", "import-scripts", "require", "async-node", "import"])
		.or(z.string());
}

function wasmLoadingType() {
	return z.enum(["fetch-streaming", "fetch", "async-node"]).or(z.string());
}

function libraryType() {
	return z
		.enum([
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
		])
		.or(z.string());
}

export function output() {
	return z.strictObject({
		iife: z.boolean().optional(),
		clean: z.boolean().optional(),
		assetModuleFilename: z.string().optional(),
		auxiliaryComment: z
			.string()
			.or(
				z.strictObject({
					amd: z.string().optional(),
					commonjs: z.string().optional(),
					commonjs2: z.string().optional(),
					root: z.string().optional()
				})
			)
			.optional(),
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
		publicPath: z.string().optional(),
		uniqueName: z.string().optional(),
		path: z.string().optional(),
		crossOriginLoading: z
			.literal(false)
			.or(z.enum(["anonymous", "use-credentials"]))
			.optional(),
		enabledWasmLoadingTypes: wasmLoadingType().array().optional(),
		wasmLoading: z.literal(false).or(wasmLoadingType()),
		enabledLibraryTypes: libraryType().optional(),
		globalObject: z.string().min(1).optional(),
		libraryExport: z.string().min(1).or(z.string().min(1).array()).optional(),
		libraryTarget: libraryType().optional(),
		hashFunction: z.string().or(z.function()).optional(),
		// TODO(hyf0)
		module: z.any().optional(),
		strictModuleErrorHandling: z.any().optional(),
		umdNamedDefine: z.any().optional(),
		chunkLoadingGlobal: z.any().optional(),
		trustedTypes: z.any().optional(),
		hashDigest: z.any().optional(),
		hashDigestLength: z.any().optional(),
		library: z.any().optional()
	});
}
