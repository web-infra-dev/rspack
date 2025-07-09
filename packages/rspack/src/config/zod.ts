import nodePath from "node:path";
import { createErrorMap, fromZodError } from "zod-validation-error/v4";
import * as z from "zod/v4";
import { getZodSwcLoaderOptionsSchema } from "../builtin-loader/swc/types";
import { memoize } from "../util/memoize";
import type * as t from "./types";
import { anyFunction, numberOrInfinity } from "./utils";

z.config({
	jitless: true
});

export { z };

export const getExternalsTypeSchema = memoize(
	() =>
		z.enum([
			"var",
			"module",
			"assign",
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
			"system",
			"promise",
			"import",
			"module-import",
			"script",
			"node-commonjs",
			"commonjs-import"
		]) satisfies z.ZodType<t.ExternalsType>
);

export const getRspackOptionsSchema = memoize(() => {
	const filenameTemplate = z.string() satisfies z.ZodType<t.FilenameTemplate>;

	const filename = filenameTemplate.or(
		anyFunction
	) satisfies z.ZodType<t.Filename>;

	//#region Name
	const name = z.string() satisfies z.ZodType<t.Name>;
	//#endregion

	//#region Dependencies
	const dependencies = z.array(name) satisfies z.ZodType<t.Dependencies>;
	//#endregion

	//#region Context
	const context = z.string().refine(val => nodePath.isAbsolute(val), {
		error: issue =>
			`The provided value ${JSON.stringify(issue.input)} must be an absolute path`
	}) satisfies z.ZodType<t.Context>;
	//#endregion

	//#region Mode
	const mode = z.enum([
		"development",
		"production",
		"none"
	]) satisfies z.ZodType<t.Mode>;
	//#endregion

	//#region Falsy
	const falsy = z.union([
		z.literal(false),
		z.literal(0),
		z.literal(""),
		z.null(),
		z.undefined()
	]) satisfies z.ZodType<t.Falsy>;

	//#endregion

	//#region Entry
	const publicPath = z
		.literal("auto")
		.or(filename) satisfies z.ZodType<t.PublicPath>;

	const baseUri = z.string() satisfies z.ZodType<t.BaseUri>;

	const chunkLoadingType = z
		.enum(["jsonp", "import-scripts", "require", "async-node", "import"])
		.or(z.string()) satisfies z.ZodType<t.ChunkLoadingType>;

	const chunkLoading = z
		.literal(false)
		.or(chunkLoadingType) satisfies z.ZodType<t.ChunkLoading>;

	const asyncChunks = z.boolean() satisfies z.ZodType<t.AsyncChunks>;

	const wasmLoadingType = z
		.enum(["fetch-streaming", "fetch", "async-node"])
		.or(z.string()) satisfies z.ZodType<t.WasmLoadingType>;

	const wasmLoading = z
		.literal(false)
		.or(wasmLoadingType) satisfies z.ZodType<t.WasmLoading>;

	const scriptType = z
		.enum(["text/javascript", "module"])
		.or(z.literal(false)) satisfies z.ZodType<t.ScriptType>;

	const libraryCustomUmdObject = z
		.strictObject({
			amd: z.string(),
			commonjs: z.string(),
			root: z.string().or(z.array(z.string()))
		})
		.partial() satisfies z.ZodType<t.LibraryCustomUmdObject>;

	const libraryName = z
		.string()
		.or(z.array(z.string()))
		.or(libraryCustomUmdObject) satisfies z.ZodType<t.LibraryName>;

	const libraryCustomUmdCommentObject = z
		.strictObject({
			amd: z.string(),
			commonjs: z.string(),
			commonjs2: z.string(),
			root: z.string()
		})
		.partial() satisfies z.ZodType<t.LibraryCustomUmdCommentObject>;

	const amdContainer = z.string() satisfies z.ZodType<t.AmdContainer>;

	const auxiliaryComment = z
		.string()
		.or(libraryCustomUmdCommentObject) satisfies z.ZodType<t.AuxiliaryComment>;

	const libraryExport = z
		.string()
		.or(z.array(z.string())) satisfies z.ZodType<t.LibraryExport>;

	const libraryType = z
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
		.or(z.string()) satisfies z.ZodType<t.LibraryType>;

	const umdNamedDefine = z.boolean() satisfies z.ZodType<t.UmdNamedDefine>;

	const libraryOptions = z.strictObject({
		amdContainer: amdContainer.optional(),
		auxiliaryComment: auxiliaryComment.optional(),
		export: libraryExport.optional(),
		name: libraryName.optional(),
		type: libraryType,
		umdNamedDefine: umdNamedDefine.optional()
	}) satisfies z.ZodType<t.LibraryOptions>;

	const library = libraryName
		.or(libraryOptions)
		.optional() satisfies z.ZodType<t.Library>;

	const layer = z.string().or(z.null()) satisfies z.ZodType<t.Layer>;

	const entryFilename = filename satisfies z.ZodType<t.EntryFilename>;

	const entryRuntime = z
		.literal(false)
		.or(z.string()) satisfies z.ZodType<t.EntryRuntime>;

	const entryItem = z
		.string()
		.or(z.array(z.string())) satisfies z.ZodType<t.EntryItem>;

	const entryDependOn = z
		.string()
		.or(z.array(z.string())) satisfies z.ZodType<t.EntryDependOn>;

	const entryDescription = z.strictObject({
		import: entryItem,
		runtime: entryRuntime.optional(),
		publicPath: publicPath.optional(),
		baseUri: baseUri.optional(),
		chunkLoading: chunkLoading.optional(),
		asyncChunks: asyncChunks.optional(),
		wasmLoading: wasmLoading.optional(),
		filename: entryFilename.optional(),
		library: libraryOptions.optional(),
		dependOn: entryDependOn.optional(),
		layer: layer.optional()
	}) satisfies z.ZodType<t.EntryDescription>;

	const entryUnnamed = entryItem satisfies z.ZodType<t.EntryUnnamed>;

	const entryObject = z.record(
		z.string(),
		entryItem.or(entryDescription)
	) satisfies z.ZodType<t.EntryObject>;

	const entryStatic = entryObject.or(
		entryUnnamed
	) satisfies z.ZodType<t.EntryStatic>;

	const entryDynamic = anyFunction satisfies z.ZodType<t.EntryDynamic>;

	const entry = entryStatic.or(entryDynamic) satisfies z.ZodType<t.Entry>;
	//#endregion

	//#region Output
	const path = z.string() satisfies z.ZodType<t.Path>;

	const pathinfo = z
		.boolean()
		.or(z.literal("verbose")) satisfies z.ZodType<t.Pathinfo>;

	const assetModuleFilename =
		filename satisfies z.ZodType<t.AssetModuleFilename>;

	const webassemblyModuleFilename =
		z.string() satisfies z.ZodType<t.WebassemblyModuleFilename>;

	const chunkFilename = filename satisfies z.ZodType<t.ChunkFilename>;

	const crossOriginLoading = z
		.literal(false)
		.or(
			z.enum(["anonymous", "use-credentials"])
		) satisfies z.ZodType<t.CrossOriginLoading>;

	const cssFilename = filename satisfies z.ZodType<t.CssFilename>;

	const cssChunkFilename = filename satisfies z.ZodType<t.CssChunkFilename>;

	const hotUpdateChunkFilename =
		filenameTemplate satisfies z.ZodType<t.HotUpdateChunkFilename>;

	const hotUpdateMainFilename =
		filenameTemplate satisfies z.ZodType<t.HotUpdateMainFilename>;

	const hotUpdateGlobal = z.string() satisfies z.ZodType<t.HotUpdateGlobal>;

	const uniqueName = z.string() satisfies z.ZodType<t.UniqueName>;

	const chunkLoadingGlobal =
		z.string() satisfies z.ZodType<t.ChunkLoadingGlobal>;

	const enabledLibraryTypes = z.array(
		libraryType
	) satisfies z.ZodType<t.EnabledLibraryTypes>;

	const clean = z.union([
		z.boolean(),
		z
			.strictObject({
				keep: z.instanceof(RegExp).or(z.string()).or(anyFunction)
			})
			.partial()
	]) satisfies z.ZodType<t.Clean>;

	const outputModule = z.boolean() satisfies z.ZodType<t.OutputModule>;

	const strictModuleExceptionHandling =
		z.boolean() satisfies z.ZodType<t.StrictModuleExceptionHandling>;

	const strictModuleErrorHandling =
		z.boolean() satisfies z.ZodType<t.StrictModuleErrorHandling>;

	const globalObject = z.string() satisfies z.ZodType<t.GlobalObject>;

	const enabledWasmLoadingTypes = z.array(
		wasmLoadingType
	) satisfies z.ZodType<t.EnabledWasmLoadingTypes>;

	const importFunctionName =
		z.string() satisfies z.ZodType<t.ImportFunctionName>;

	const importMetaName = z.string() satisfies z.ZodType<t.ImportMetaName>;

	const iife = z.boolean() satisfies z.ZodType<t.Iife>;

	const enabledChunkLoadingTypes = z.array(
		chunkLoadingType
	) satisfies z.ZodType<t.EnabledChunkLoadingTypes>;

	const chunkFormat = z
		.literal(false)
		.or(z.string()) satisfies z.ZodType<t.ChunkFormat>;

	const workerPublicPath = z.string() satisfies z.ZodType<t.WorkerPublicPath>;

	const trustedTypes = z
		.strictObject({
			policyName: z.string(),
			onPolicyCreationFailure: z.enum(["continue", "stop"])
		})
		.partial() satisfies z.ZodType<t.TrustedTypes>;

	const hashDigest = z.string() satisfies z.ZodType<t.HashDigest>;

	const hashDigestLength = z.int() satisfies z.ZodType<t.HashDigestLength>;

	const hashFunction = z.enum([
		"md4",
		"xxhash64",
		"sha256"
	]) satisfies z.ZodType<t.HashFunction>;

	const hashSalt = z.string() satisfies z.ZodType<t.HashSalt>;

	const sourceMapFilename = z.string() satisfies z.ZodType<t.SourceMapFilename>;

	const devtoolNamespace = z.string() satisfies z.ZodType<t.DevtoolNamespace>;

	const devtoolModuleFilenameTemplate = z.union([
		z.string(),
		anyFunction
	]) satisfies z.ZodType<t.DevtoolModuleFilenameTemplate>;

	const devtoolFallbackModuleFilenameTemplate =
		devtoolModuleFilenameTemplate satisfies z.ZodType<t.DevtoolFallbackModuleFilenameTemplate>;

	const environment = z
		.strictObject({
			arrowFunction: z.boolean(),
			asyncFunction: z.boolean(),
			bigIntLiteral: z.boolean(),
			const: z.boolean(),
			destructuring: z.boolean(),
			document: z.boolean(),
			dynamicImport: z.boolean(),
			dynamicImportInWorker: z.boolean(),
			forOf: z.boolean(),
			globalThis: z.boolean(),
			module: z.boolean(),
			nodePrefixForCoreModules: z.boolean(),
			optionalChaining: z.boolean(),
			templateLiteral: z.boolean()
		})
		.partial() satisfies z.ZodType<t.Environment>;

	const output = z
		.strictObject({
			path: path,
			pathinfo: pathinfo,
			clean: clean,
			publicPath: publicPath,
			filename: filename,
			chunkFilename: chunkFilename,
			crossOriginLoading: crossOriginLoading,
			cssFilename: cssFilename,
			cssHeadDataCompression: z.boolean(),
			cssChunkFilename: cssChunkFilename,
			hotUpdateMainFilename: hotUpdateMainFilename,
			hotUpdateChunkFilename: hotUpdateChunkFilename,
			hotUpdateGlobal: hotUpdateGlobal,
			assetModuleFilename: assetModuleFilename,
			uniqueName: uniqueName,
			chunkLoadingGlobal: chunkLoadingGlobal,
			enabledLibraryTypes: enabledLibraryTypes,
			library: library,
			libraryExport: libraryExport,
			libraryTarget: libraryType,
			umdNamedDefine: umdNamedDefine,
			auxiliaryComment: auxiliaryComment,
			module: outputModule,
			strictModuleExceptionHandling: strictModuleExceptionHandling,
			strictModuleErrorHandling: strictModuleErrorHandling,
			globalObject: globalObject,
			importFunctionName: importFunctionName,
			importMetaName: importMetaName,
			iife: iife,
			wasmLoading: wasmLoading,
			enabledWasmLoadingTypes: enabledWasmLoadingTypes,
			webassemblyModuleFilename: webassemblyModuleFilename,
			chunkFormat: chunkFormat,
			chunkLoading: chunkLoading,
			enabledChunkLoadingTypes: enabledChunkLoadingTypes,
			trustedTypes: z.literal(true).or(z.string()).or(trustedTypes),
			sourceMapFilename: sourceMapFilename,
			hashDigest: hashDigest,
			hashDigestLength: hashDigestLength,
			hashFunction: hashFunction,
			hashSalt: hashSalt,
			asyncChunks: asyncChunks,
			workerChunkLoading: chunkLoading,
			workerWasmLoading: wasmLoading,
			workerPublicPath: workerPublicPath,
			scriptType: scriptType,
			devtoolNamespace: devtoolNamespace,
			devtoolModuleFilenameTemplate: devtoolModuleFilenameTemplate,
			devtoolFallbackModuleFilenameTemplate:
				devtoolFallbackModuleFilenameTemplate,
			chunkLoadTimeout: numberOrInfinity,
			charset: z.boolean(),
			environment: environment,
			compareBeforeEmit: z.boolean()
		})
		.partial() satisfies z.ZodType<t.Output>;
	//#endregion

	//#region Resolve
	const resolveAlias = z
		.record(
			z.string(),
			z
				.literal(false)
				.or(z.string())
				.or(z.array(z.string().or(z.literal(false))))
		)
		.or(z.literal(false)) satisfies z.ZodType<t.ResolveAlias>;

	const resolveTsConfigFile = z.string();
	const resolveTsConfig = resolveTsConfigFile.or(
		z.strictObject({
			configFile: resolveTsConfigFile,
			references: z.array(z.string()).or(z.literal("auto")).optional()
		})
	) satisfies z.ZodType<t.ResolveTsConfig>;

	const baseResolveOptions = z
		.strictObject({
			alias: resolveAlias,
			conditionNames: z.array(z.string()),
			extensions: z.array(z.string()),
			fallback: resolveAlias,
			mainFields: z.array(z.string()),
			mainFiles: z.array(z.string()),
			modules: z.array(z.string()),
			preferRelative: z.boolean(),
			preferAbsolute: z.boolean(),
			symlinks: z.boolean(),
			enforceExtension: z.boolean(),
			importsFields: z.array(z.string()),
			descriptionFiles: z.array(z.string()),
			tsConfig: resolveTsConfig,
			fullySpecified: z.boolean(),
			exportsFields: z.array(z.string()),
			extensionAlias: z.record(z.string(), z.string().or(z.array(z.string()))),
			aliasFields: z.array(z.string()),
			restrictions: z.array(z.string()),
			roots: z.array(z.string()),
			pnp: z.boolean()
		})
		.partial() satisfies z.ZodType<t.ResolveOptions>;

	const resolveOptions: z.ZodType<t.ResolveOptions> = baseResolveOptions.extend(
		{
			byDependency: z
				.lazy(() => z.record(z.string(), resolveOptions))
				.optional()
		}
	);

	//#endregion

	//#region Module
	const baseRuleSetCondition = z
		.instanceof(RegExp)
		.or(z.string())
		.or(anyFunction);

	const ruleSetCondition: z.ZodType<t.RuleSetCondition> = baseRuleSetCondition
		.or(z.lazy(() => ruleSetConditions))
		.or(z.lazy(() => ruleSetLogicalConditions));

	const ruleSetConditions: z.ZodType<t.RuleSetConditions> = z.lazy(() =>
		z.array(ruleSetCondition)
	);

	const ruleSetLogicalConditions: z.ZodType<t.RuleSetLogicalConditions> = z
		.strictObject({
			and: ruleSetConditions,
			or: ruleSetConditions,
			not: ruleSetCondition
		})
		.partial();

	const ruleSetLoader = z.string() satisfies z.ZodType<t.RuleSetLoader>;

	const ruleSetLoaderOptions = z
		.string()
		.or(
			z.record(z.string(), z.any())
		) satisfies z.ZodType<t.RuleSetLoaderOptions>;

	const ruleSetLoaderWithOptions = z.strictObject({
		ident: z.string().optional(),
		loader: ruleSetLoader,
		options: ruleSetLoaderOptions.optional(),
		parallel: z.boolean().optional()
	}) satisfies z.ZodType<t.RuleSetLoaderWithOptions>;

	const builtinSWCLoaderChecker: z.core.CheckFn<
		t.RuleSetLoaderWithOptions | t.RuleSetRule | undefined
	> = ctx => {
		const data = ctx.value;
		if (
			data?.loader !== "builtin:swc-loader" ||
			typeof data?.options !== "object"
		) {
			return;
		}

		const res = getZodSwcLoaderOptionsSchema().safeParse(data.options);

		if (!res.success) {
			const validationErr = fromZodError(res.error, {
				prefix: "Invalid options for 'builtin:swc-loader'",
				error: createErrorMap({
					issuesInTitleCase: false
				})
			});
			ctx.issues.push({
				code: "custom",
				message: validationErr.message,
				input: data.options
			});
		}
	};

	const ruleSetUseItem = ruleSetLoader.or(
		ruleSetLoaderWithOptions.check(builtinSWCLoaderChecker)
	) satisfies z.ZodType<t.RuleSetUseItem>;

	const ruleSetUse = ruleSetUseItem
		.or(ruleSetUseItem.array())
		.or(anyFunction) satisfies z.ZodType<t.RuleSetUse>;

	const baseRuleSetRule = z
		.strictObject({
			test: ruleSetCondition,
			exclude: ruleSetCondition,
			include: ruleSetCondition,
			issuer: ruleSetCondition,
			issuerLayer: ruleSetCondition,
			dependency: ruleSetCondition,
			resource: ruleSetCondition,
			resourceFragment: ruleSetCondition,
			resourceQuery: ruleSetCondition,
			scheme: ruleSetCondition,
			mimetype: ruleSetCondition,
			descriptionData: z.record(z.string(), ruleSetCondition),
			with: z.record(z.string(), ruleSetCondition),
			type: z.string(),
			layer: z.string(),
			loader: ruleSetLoader,
			options: ruleSetLoaderOptions,
			use: ruleSetUse,
			parser: z.record(z.string(), z.any()),
			generator: z.record(z.string(), z.any()),
			resolve: resolveOptions,
			sideEffects: z.boolean(),
			enforce: z.literal("pre").or(z.literal("post"))
		})
		.partial() satisfies z.ZodType<t.RuleSetRule>;

	const extendedBaseRuleSetRule: z.ZodType<t.RuleSetRule> =
		baseRuleSetRule.extend({
			oneOf: z.lazy(() => ruleSetRule.or(falsy).array()).optional(),
			rules: z.lazy(() => ruleSetRule.or(falsy).array()).optional()
		}) satisfies z.ZodType<t.RuleSetRule>;

	const ruleSetRule = extendedBaseRuleSetRule.check(builtinSWCLoaderChecker);

	const ruleSetRules = z.array(
		z.literal("...").or(ruleSetRule).or(falsy)
	) satisfies z.ZodType<t.RuleSetRules>;

	const assetParserDataUrlOptions = z
		.strictObject({
			maxSize: numberOrInfinity
		})
		.partial() satisfies z.ZodType<t.AssetParserDataUrlOptions>;

	const assetParserDataUrl =
		assetParserDataUrlOptions satisfies z.ZodType<t.AssetParserDataUrl>;

	const assetParserOptions = z
		.strictObject({
			dataUrlCondition: assetParserDataUrl
		})
		.partial() satisfies z.ZodType<t.AssetParserOptions>;

	const cssParserNamedExports =
		z.boolean() satisfies z.ZodType<t.CssParserNamedExports>;

	const cssParserUrl = z.boolean() satisfies z.ZodType<t.CssParserUrl>;

	const cssParserOptions = z
		.strictObject({
			namedExports: cssParserNamedExports,
			url: cssParserUrl
		})
		.partial() satisfies z.ZodType<t.CssParserOptions>;

	const cssAutoParserOptions = z
		.strictObject({
			namedExports: cssParserNamedExports,
			url: cssParserUrl
		})
		.partial() satisfies z.ZodType<t.CssAutoParserOptions>;

	const cssModuleParserOptions = z
		.strictObject({
			namedExports: cssParserNamedExports,
			url: cssParserUrl
		})
		.partial() satisfies z.ZodType<t.CssModuleParserOptions>;

	const dynamicImportMode = z.enum(["eager", "lazy", "weak", "lazy-once"]);
	const dynamicImportPreload = z.union([z.boolean(), numberOrInfinity]);
	const dynamicImportPrefetch = z.union([z.boolean(), numberOrInfinity]);
	const dynamicImportFetchPriority = z.enum(["low", "high", "auto"]);
	const javascriptParserUrl = z.union([z.literal("relative"), z.boolean()]);
	const exprContextCritical = z.boolean();
	const wrappedContextCritical = z.boolean();
	const wrappedContextRegExp = z.instanceof(RegExp);
	const exportsPresence = z
		.enum(["error", "warn", "auto"])
		.or(z.literal(false));
	const importExportsPresence = z
		.enum(["error", "warn", "auto"])
		.or(z.literal(false));
	const reexportExportsPresence = z
		.enum(["error", "warn", "auto"])
		.or(z.literal(false));
	const strictExportPresence = z.boolean();
	const worker = z.array(z.string()).or(z.boolean());
	const overrideStrict = z.enum(["strict", "non-strict"]);
	const requireAsExpression = z.boolean();
	const requireDynamic = z.boolean();
	const requireResolve = z.boolean();
	const importDynamic = z.boolean();
	const inlineConst = z.boolean();
	const typeReexportsPresence = z.enum([
		"no-tolerant",
		"tolerant",
		"tolerant-no-check"
	]);

	const javascriptParserOptions = z
		.strictObject({
			dynamicImportMode: dynamicImportMode,
			dynamicImportPreload: dynamicImportPreload,
			dynamicImportPrefetch: dynamicImportPrefetch,
			dynamicImportFetchPriority: dynamicImportFetchPriority,
			importMeta: z.boolean(),
			url: javascriptParserUrl,
			exprContextCritical: exprContextCritical,
			wrappedContextCritical: wrappedContextCritical,
			wrappedContextRegExp: wrappedContextRegExp,
			exportsPresence: exportsPresence,
			importExportsPresence: importExportsPresence,
			reexportExportsPresence: reexportExportsPresence,
			strictExportPresence: strictExportPresence,
			worker: worker,
			overrideStrict: overrideStrict,
			// #region Not available in webpack yet.
			requireAsExpression: requireAsExpression,
			requireDynamic: requireDynamic,
			requireResolve: requireResolve,
			importDynamic: importDynamic,
			inlineConst: inlineConst,
			typeReexportsPresence: typeReexportsPresence
			// #endregion
		})
		.partial() satisfies z.ZodType<t.JavascriptParserOptions>;

	const parserOptionsByModuleTypeKnown = z
		.strictObject({
			asset: assetParserOptions,
			css: cssParserOptions,
			"css/auto": cssAutoParserOptions,
			"css/module": cssModuleParserOptions,
			javascript: javascriptParserOptions,
			"javascript/auto": javascriptParserOptions,
			"javascript/dynamic": javascriptParserOptions,
			"javascript/esm": javascriptParserOptions
		})
		.partial() satisfies z.ZodType<t.ParserOptionsByModuleTypeKnown>;

	const parserOptionsByModuleType = parserOptionsByModuleTypeKnown;

	const assetGeneratorDataUrlOptions = z
		.strictObject({
			encoding: z.literal(false).or(z.literal("base64")),
			mimetype: z.string()
		})
		.partial() satisfies z.ZodType<t.AssetGeneratorDataUrlOptions>;

	const assetGeneratorDataUrlFunction =
		anyFunction satisfies z.ZodType<t.AssetGeneratorDataUrlFunction>;

	const assetGeneratorDataUrl = assetGeneratorDataUrlOptions.or(
		assetGeneratorDataUrlFunction
	) satisfies z.ZodType<t.AssetGeneratorDataUrl>;

	const assetInlineGeneratorOptions = z
		.strictObject({
			dataUrl: assetGeneratorDataUrl
		})
		.partial() satisfies z.ZodType<t.AssetInlineGeneratorOptions>;

	const assetResourceGeneratorOptions = z
		.strictObject({
			emit: z.boolean(),
			filename: filename,
			publicPath: publicPath,
			outputPath: filename
		})
		.partial() satisfies z.ZodType<t.AssetResourceGeneratorOptions>;

	const assetGeneratorOptions = assetInlineGeneratorOptions.merge(
		assetResourceGeneratorOptions
	) satisfies z.ZodType<t.AssetGeneratorOptions>;

	const cssGeneratorExportsConvention = z.enum([
		"as-is",
		"camel-case",
		"camel-case-only",
		"dashes",
		"dashes-only"
	]) satisfies z.ZodType<t.CssGeneratorExportsConvention>;

	const cssGeneratorExportsOnly =
		z.boolean() satisfies z.ZodType<t.CssGeneratorExportsOnly>;

	const cssGeneratorLocalIdentName =
		z.string() satisfies z.ZodType<t.CssGeneratorLocalIdentName>;

	const cssGeneratorEsModule =
		z.boolean() satisfies z.ZodType<t.CssGeneratorEsModule>;

	const cssGeneratorOptions = z
		.strictObject({
			exportsOnly: cssGeneratorExportsOnly,
			esModule: cssGeneratorEsModule
		})
		.partial() satisfies z.ZodType<t.CssGeneratorOptions>;

	const cssAutoGeneratorOptions = z
		.strictObject({
			exportsConvention: cssGeneratorExportsConvention,
			exportsOnly: cssGeneratorExportsOnly,
			localIdentName: cssGeneratorLocalIdentName,
			esModule: cssGeneratorEsModule
		})
		.partial() satisfies z.ZodType<t.CssAutoGeneratorOptions>;

	const cssModuleGeneratorOptions = z
		.strictObject({
			exportsConvention: cssGeneratorExportsConvention,
			exportsOnly: cssGeneratorExportsOnly,
			localIdentName: cssGeneratorLocalIdentName,
			esModule: cssGeneratorEsModule
		})
		.partial() satisfies z.ZodType<t.CssModuleGeneratorOptions>;

	const jsonGeneratorOptions = z
		.strictObject({
			JSONParse: z.boolean()
		})
		.partial() satisfies z.ZodType<t.JsonGeneratorOptions>;

	const generatorOptionsByModuleTypeKnown = z
		.strictObject({
			asset: assetGeneratorOptions,
			"asset/inline": assetInlineGeneratorOptions,
			"asset/resource": assetResourceGeneratorOptions,
			css: cssGeneratorOptions,
			"css/auto": cssAutoGeneratorOptions,
			"css/module": cssModuleGeneratorOptions,
			json: jsonGeneratorOptions
		})
		.partial() satisfies z.ZodType<t.GeneratorOptionsByModuleTypeKnown>;

	const generatorOptionsByModuleType = generatorOptionsByModuleTypeKnown;

	const noParseOptionSingle = z
		.string()
		.or(z.instanceof(RegExp))
		.or(anyFunction);
	const noParseOption = noParseOptionSingle.or(
		z.array(noParseOptionSingle)
	) satisfies z.ZodType<t.NoParseOption>;

	const moduleOptions = z
		.strictObject({
			defaultRules: ruleSetRules,
			rules: ruleSetRules,
			parser: parserOptionsByModuleType,
			generator: generatorOptionsByModuleType,
			noParse: noParseOption
		})
		.partial() satisfies z.ZodType<t.ModuleOptions>;
	//#endregion

	//#region Target
	const allowTarget = z.union([
		z.enum([
			"web",
			"webworker",
			"es3",
			"es5",
			"es2015",
			"es2016",
			"es2017",
			"es2018",
			"es2019",
			"es2020",
			"es2021",
			"es2022",
			"es2023",
			"es2024",
			"es2025"
		]),
		z.literal("node"),
		z.literal("async-node"),
		z.custom<`node${number}`>(
			value => typeof value === "string" && /^node\d+$/.test(value)
		),
		z.custom<`async-node${number}`>(
			value => typeof value === "string" && /^async-node\d+$/.test(value)
		),
		z.custom<`node${number}.${number}`>(
			value => typeof value === "string" && /^node\d+\.\d+$/.test(value)
		),
		z.custom<`async-node${number}.${number}`>(
			value => typeof value === "string" && /^async-node\d+\.\d+$/.test(value)
		),
		z.literal("electron-main"),
		z.custom<`electron${number}-main`>(
			value => typeof value === "string" && /^electron\d+-main$/.test(value)
		),
		z.custom<`electron${number}.${number}-main`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-main$/.test(value)
		),
		z.literal("electron-renderer"),
		z.custom<`electron${number}-renderer`>(
			value => typeof value === "string" && /^electron\d+-renderer$/.test(value)
		),
		z.custom<`electron${number}.${number}-renderer`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-renderer$/.test(value)
		),
		z.literal("electron-preload"),
		z.custom<`electron${number}-preload`>(
			value => typeof value === "string" && /^electron\d+-preload$/.test(value)
		),
		z.custom<`electron${number}.${number}-preload`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-preload$/.test(value)
		),
		z.literal("nwjs"),
		z.custom<`nwjs${number}`>(
			value => typeof value === "string" && /^nwjs\d+$/.test(value)
		),
		z.custom<`nwjs${number}.${number}`>(
			value => typeof value === "string" && /^nwjs\d+\.\d+$/.test(value)
		),
		z.literal("node-webkit"),
		z.custom<`node-webkit${number}`>(
			value => typeof value === "string" && /^node-webkit\d+$/.test(value)
		),
		z.custom<`node-webkit${number}.${number}`>(
			value => typeof value === "string" && /^node-webkit\d+\.\d+$/.test(value)
		),
		z.literal("browserslist"),
		z.custom<`browserslist:${string}`>(
			value => typeof value === "string" && /^browserslist:(.+)$/.test(value)
		)
	]);

	const target = z.union([
		z.literal(false),
		allowTarget,
		allowTarget.array()
	]) satisfies z.ZodType<t.Target>;
	//#endregion

	const externalItemObjectValue = z.record(
		z.string(),
		z.string().or(z.string().array())
	) satisfies z.ZodType<t.ExternalItemObjectValue>;

	const externalItemUmdValue = z.strictObject({
		root: z.string().or(z.string().array()),
		commonjs: z.string().or(z.string().array()),
		commonjs2: z.string().or(z.string().array()),
		amd: z.string().or(z.string().array())
	}) satisfies z.ZodType<t.ExternalItemUmdValue>;

	const externalUmdChecker: z.core.CheckFn<t.RspackOptions> = ctx => {
		let isLibraryUmd = false;
		const config = ctx.value;
		const library = config?.output?.library;
		if (typeof library === "object" && "type" in library) {
			isLibraryUmd = library.type === "umd";
		} else {
			isLibraryUmd = config?.output?.libraryTarget === "umd";
		}

		if (!isLibraryUmd) {
			return;
		}

		if (
			config?.externalsType !== undefined &&
			config?.externalsType !== "umd"
		) {
			return;
		}

		if (!Array.isArray(config?.externals)) {
			checkExternalItem(config?.externals, ["externals"]);
		} else {
			config.externals.forEach((external, index) =>
				checkExternalItem(external, ["externals", index])
			);
		}

		function checkExternalItem(
			externalItem: t.ExternalItem | undefined,
			path: (string | number)[]
		) {
			if (typeof externalItem === "object" && externalItem !== null) {
				for (const [key, value] of Object.entries(externalItem)) {
					checkExternalItemValue(value, [...path, key]);
				}
			}
		}

		function checkExternalItemValue(
			externalItemValue: t.ExternalItemValue | undefined,
			path: (string | number)[]
		) {
			if (typeof externalItemValue === "object" && externalItemValue !== null) {
				const result = externalItemUmdValue.safeParse(externalItemValue);
				if (!result.success) {
					ctx.issues.push({
						code: "custom",
						message: `External object must have "root", "commonjs", "commonjs2", "amd" properties when "libraryType" or "externalsType" is "umd"`,
						input: externalItemValue,
						path
					});
				}
				return;
			}
		}
	};

	// #region Externals
	const externalItemValue = z
		.string()
		.or(z.boolean())
		.or(z.string().array().min(1))
		.or(externalItemObjectValue) satisfies z.ZodType<t.ExternalItemValue>;

	const externalItemObjectUnknown = z.record(
		z.string(),
		externalItemValue
	) satisfies z.ZodType<t.ExternalItemObjectUnknown>;

	const externalItem = z
		.string()
		.or(z.instanceof(RegExp))
		.or(externalItemObjectUnknown)
		.or(anyFunction) satisfies z.ZodType<t.ExternalItem>;

	const externals = externalItem
		.array()
		.or(externalItem) satisfies z.ZodType<t.Externals>;

	//#region ExternalsPresets
	const externalsPresets = z
		.strictObject({
			node: z.boolean(),
			web: z.boolean(),
			webAsync: z.boolean(),
			electron: z.boolean(),
			electronMain: z.boolean(),
			electronPreload: z.boolean(),
			electronRenderer: z.boolean(),
			nwjs: z.boolean()
		})
		.partial() satisfies z.ZodType<t.ExternalsPresets>;
	//#endregion

	//#region InfrastructureLogging
	const filterItemTypes = z
		.instanceof(RegExp)
		.or(z.string())
		.or(anyFunction) satisfies z.ZodType<t.FilterItemTypes>;

	const filterTypes = filterItemTypes
		.array()
		.or(filterItemTypes) satisfies z.ZodType<t.FilterTypes>;

	const infrastructureLogging = z
		.strictObject({
			appendOnly: z.boolean(),
			colors: z.boolean(),
			console: z.custom<Console>(),
			debug: z.boolean().or(filterTypes),
			level: z.enum(["none", "error", "warn", "info", "log", "verbose"]),
			stream: z.custom<NodeJS.WritableStream>()
		})
		.partial() satisfies z.ZodType<t.InfrastructureLogging>;
	//#endregion

	//#region DevTool
	const devTool = z
		.literal(false)
		.or(z.literal("eval"))
		.or(
			z.string().refine(
				val => {
					// Pattern: [inline-|hidden-|eval-][nosources-][cheap-[module-]]source-map[-debugids]
					const pattern =
						/^(inline-|hidden-|eval-)?(nosources-)?(cheap-(module-)?)?source-map(-debugids)?$/;
					return pattern.test(val);
				},
				{
					error:
						"Expect value to match the pattern: [inline-|hidden-|eval-][nosources-][cheap-[module-]]source-map[-debugids]"
				}
			) as z.ZodType<t.DevTool>
		) satisfies z.ZodType<t.DevTool>;
	//#endregion

	//#region Node
	const nodeOptions = z
		.strictObject({
			__dirname: z
				.boolean()
				.or(z.enum(["warn-mock", "mock", "eval-only", "node-module"])),
			__filename: z
				.boolean()
				.or(z.enum(["warn-mock", "mock", "eval-only", "node-module"])),
			global: z.boolean().or(z.literal("warn"))
		})
		.partial() satisfies z.ZodType<t.NodeOptions>;

	const node = z.literal(false).or(nodeOptions) satisfies z.ZodType<t.Node>;

	const loader = z.record(z.string(), z.any()) satisfies z.ZodType<t.Loader>;
	//#endregion

	//#region Snapshot
	const snapshotOptions = z.strictObject(
		{}
	) satisfies z.ZodType<t.SnapshotOptions>;
	//#endregion

	//#region Cache
	const cacheOptions = z.boolean() satisfies z.ZodType<t.CacheOptions>;
	//#endregion

	//#region Stats
	const statsPresets = z.enum([
		"normal",
		"none",
		"verbose",
		"errors-only",
		"errors-warnings",
		"minimal",
		"detailed",
		"summary"
	]);
	const statsOptions = z
		.strictObject({
			all: z.boolean(),
			preset: z.boolean().or(statsPresets),
			assets: z.boolean(),
			chunks: z.boolean(),
			modules: z.boolean(),
			entrypoints: z.boolean().or(z.literal("auto")),
			chunkGroups: z.boolean(),
			warnings: z.boolean(),
			warningsCount: z.boolean(),
			errors: z.boolean(),
			errorsCount: z.boolean(),
			colors: z.boolean(),
			hash: z.boolean(),
			version: z.boolean(),
			reasons: z.boolean(),
			publicPath: z.boolean(),
			outputPath: z.boolean(),
			chunkModules: z.boolean(),
			chunkRelations: z.boolean(),
			ids: z.boolean(),
			timings: z.boolean(),
			builtAt: z.boolean(),
			moduleAssets: z.boolean(),
			nestedModules: z.boolean(),
			source: z.boolean(),
			logging: z
				.enum(["none", "error", "warn", "info", "log", "verbose"])
				.or(z.boolean()),
			loggingDebug: z.boolean().or(filterTypes),
			loggingTrace: z.boolean(),
			runtimeModules: z.boolean(),
			children: z.boolean(),
			usedExports: z.boolean(),
			providedExports: z.boolean(),
			optimizationBailout: z.boolean(),
			groupModulesByType: z.boolean(),
			groupModulesByCacheStatus: z.boolean(),
			groupModulesByLayer: z.boolean(),
			groupModulesByAttributes: z.boolean(),
			groupModulesByPath: z.boolean(),
			groupModulesByExtension: z.boolean(),
			modulesSpace: z.int(),
			chunkModulesSpace: z.int(),
			nestedModulesSpace: z.int(),
			relatedAssets: z.boolean(),
			groupAssetsByEmitStatus: z.boolean(),
			groupAssetsByInfo: z.boolean(),
			groupAssetsByPath: z.boolean(),
			groupAssetsByExtension: z.boolean(),
			groupAssetsByChunk: z.boolean(),
			assetsSpace: z.int(),
			orphanModules: z.boolean(),
			excludeModules: z
				.array(z.string().or(z.instanceof(RegExp)).or(anyFunction))
				.or(z.string())
				.or(z.instanceof(RegExp))
				.or(anyFunction)
				.or(z.boolean()),
			excludeAssets: z
				.array(z.string().or(z.instanceof(RegExp)).or(anyFunction))
				.or(z.string())
				.or(z.instanceof(RegExp))
				.or(anyFunction),
			modulesSort: z.string(),
			chunkModulesSort: z.string(),
			nestedModulesSort: z.string(),
			chunksSort: z.string(),
			assetsSort: z.string(),
			performance: z.boolean(),
			env: z.boolean(),
			chunkGroupAuxiliary: z.boolean(),
			chunkGroupChildren: z.boolean(),
			chunkGroupMaxAssets: numberOrInfinity,
			dependentModules: z.boolean(),
			chunkOrigins: z.boolean(),
			runtime: z.boolean(),
			depth: z.boolean(),
			reasonsSpace: z.int(),
			groupReasonsByOrigin: z.boolean(),
			errorDetails: z.boolean(),
			errorStack: z.boolean(),
			moduleTrace: z.boolean(),
			cachedModules: z.boolean(),
			cachedAssets: z.boolean(),
			cached: z.boolean(),
			errorsSpace: z.int(),
			warningsSpace: z.int()
		})
		.partial() satisfies z.ZodType<t.StatsOptions>;

	const statsValue = z
		.boolean()
		.or(statsPresets)
		.or(statsOptions) satisfies z.ZodType<t.StatsValue>;
	//#endregion

	//#region Plugins
	const plugin = z.union([
		z.custom<
			| t.RspackPluginInstance
			| t.RspackPluginFunction
			| t.WebpackPluginInstance
			| t.WebpackPluginFunction
		>(),
		falsy
	]) satisfies z.ZodType<t.Plugin>;

	const plugins = plugin.array() satisfies z.ZodType<t.Plugins>;
	//#endregion

	//#region Optimization
	const optimizationRuntimeChunk = z
		.enum(["single", "multiple"])
		.or(z.boolean())
		.or(
			z
				.strictObject({
					name: z.string().or(anyFunction)
				})
				.partial()
		) satisfies z.ZodType<t.OptimizationRuntimeChunk>;

	const optimizationSplitChunksNameFunction =
		anyFunction satisfies z.ZodType<t.OptimizationSplitChunksNameFunction>;

	const optimizationSplitChunksName = z
		.string()
		.or(z.literal(false))
		.or(optimizationSplitChunksNameFunction);
	const optimizationSplitChunksChunks = z
		.enum(["initial", "async", "all"])
		.or(z.instanceof(RegExp))
		.or(anyFunction);
	const optimizationSplitChunksSizes = numberOrInfinity.or(
		z.record(z.string(), numberOrInfinity)
	);
	const optimizationSplitChunksDefaultSizeTypes = z.array(z.string());

	const sharedOptimizationSplitChunksCacheGroup = {
		chunks: optimizationSplitChunksChunks,
		defaultSizeTypes: optimizationSplitChunksDefaultSizeTypes,
		minChunks: z.number().min(1).or(z.literal(Number.POSITIVE_INFINITY)),
		usedExports: z.boolean(),
		name: optimizationSplitChunksName,
		filename: filename,
		minSize: optimizationSplitChunksSizes,
		minSizeReduction: optimizationSplitChunksSizes,
		maxSize: optimizationSplitChunksSizes,
		maxAsyncSize: optimizationSplitChunksSizes,
		maxInitialSize: optimizationSplitChunksSizes,
		maxAsyncRequests: numberOrInfinity,
		maxInitialRequests: numberOrInfinity,
		automaticNameDelimiter: z.string()
	};

	const optimizationSplitChunksCacheGroup = z
		.strictObject({
			test: z.string().or(z.instanceof(RegExp)).or(anyFunction),
			priority: numberOrInfinity,
			enforce: z.boolean(),
			reuseExistingChunk: z.boolean(),
			type: z.string().or(z.instanceof(RegExp)),
			idHint: z.string(),
			layer: z.string().or(z.instanceof(RegExp)).or(anyFunction),
			...sharedOptimizationSplitChunksCacheGroup
		})
		.partial() satisfies z.ZodType<t.OptimizationSplitChunksCacheGroup>;

	const optimizationSplitChunksOptions = z
		.strictObject({
			cacheGroups: z.record(
				z.string(),
				z.literal(false).or(optimizationSplitChunksCacheGroup)
			),
			fallbackCacheGroup: z
				.strictObject({
					chunks: optimizationSplitChunksChunks,
					minSize: numberOrInfinity,
					maxSize: numberOrInfinity,
					maxAsyncSize: numberOrInfinity,
					maxInitialSize: numberOrInfinity,
					automaticNameDelimiter: z.string()
				})
				.partial(),
			hidePathInfo: z.boolean(),
			...sharedOptimizationSplitChunksCacheGroup
		})
		.partial() satisfies z.ZodType<t.OptimizationSplitChunksOptions>;

	const optimization = z
		.strictObject({
			moduleIds: z.enum(["named", "natural", "deterministic"]),
			chunkIds: z.enum([
				"natural",
				"named",
				"deterministic",
				"size",
				"total-size"
			]),
			minimize: z.boolean(),
			minimizer: z.literal("...").or(plugin).array(),
			mergeDuplicateChunks: z.boolean(),
			splitChunks: z.literal(false).or(optimizationSplitChunksOptions),
			runtimeChunk: optimizationRuntimeChunk,
			removeAvailableModules: z.boolean(),
			removeEmptyChunks: z.boolean(),
			realContentHash: z.boolean(),
			sideEffects: z.enum(["flag"]).or(z.boolean()),
			providedExports: z.boolean(),
			concatenateModules: z.boolean(),
			innerGraph: z.boolean(),
			usedExports: z.enum(["global"]).or(z.boolean()),
			mangleExports: z.enum(["size", "deterministic"]).or(z.boolean()),
			nodeEnv: z.union([z.string(), z.literal(false)]),
			emitOnErrors: z.boolean(),
			avoidEntryIife: z.boolean()
		})
		.partial() satisfies z.ZodType<t.Optimization>;
	//#endregion

	//#region Experiments
	const rspackFutureOptions = z
		.strictObject({
			bundlerInfo: z
				.strictObject({
					version: z.string(),
					bundler: z.string(),
					force: z.boolean().or(z.array(z.enum(["version", "uniqueId"])))
				})
				.partial()
		})
		.partial() satisfies z.ZodType<t.RspackFutureOptions>;

	const experimentCacheOptions = z
		.object({
			type: z.enum(["memory"])
		})
		.or(
			z.object({
				type: z.enum(["persistent"]),
				buildDependencies: z.string().array().optional(),
				version: z.string().optional(),
				snapshot: z
					.object({
						immutablePaths: z
							.string()
							.or(z.instanceof(RegExp))
							.array()
							.optional(),
						unmanagedPaths: z
							.string()
							.or(z.instanceof(RegExp))
							.array()
							.optional(),
						managedPaths: z.string().or(z.instanceof(RegExp)).array().optional()
					})
					.optional(),
				storage: z
					.object({
						type: z.enum(["filesystem"]),
						directory: z.string().optional()
					})
					.optional()
			})
		);

	const lazyCompilationOptions = z
		.object({
			imports: z.boolean(),
			entries: z.boolean(),
			test: z.instanceof(RegExp).or(anyFunction),
			client: z.string(),
			serverUrl: z.string(),
			prefix: z.string()
		})
		.partial() satisfies z.ZodType<t.LazyCompilationOptions>;

	const incremental = z
		.strictObject({
			make: z.boolean(),
			inferAsyncModules: z.boolean(),
			providedExports: z.boolean(),
			dependenciesDiagnostics: z.boolean(),
			sideEffects: z.boolean(),
			buildChunkGraph: z.boolean(),
			moduleIds: z.boolean(),
			chunkIds: z.boolean(),
			modulesHashes: z.boolean(),
			modulesCodegen: z.boolean(),
			modulesRuntimeRequirements: z.boolean(),
			chunksRuntimeRequirements: z.boolean(),
			chunksHashes: z.boolean(),
			chunksRender: z.boolean(),
			emitAssets: z.boolean()
		})
		.partial() satisfies z.ZodType<t.Incremental>;

	// Define buildHttp options schema
	const buildHttpOptions = z.object({
		allowedUris: z.array(z.union([z.string(), z.instanceof(RegExp)])),
		lockfileLocation: z.string().optional(),
		cacheLocation: z.union([z.string(), z.literal(false)]).optional(),
		upgrade: z.boolean().optional(),
		// proxy: z.string().optional(),
		// frozen: z.boolean().optional(),
		httpClient: anyFunction.optional()
	}) satisfies z.ZodType<t.HttpUriOptions>;

	const useInputFileSystem = z.union([
		z.literal(false),
		z.array(z.instanceof(RegExp))
	]) satisfies z.ZodType<t.UseInputFileSystem>;

	const experiments = z
		.strictObject({
			cache: z.boolean().or(experimentCacheOptions),
			lazyCompilation: z.boolean().or(lazyCompilationOptions),
			asyncWebAssembly: z.boolean(),
			outputModule: z.boolean(),
			topLevelAwait: z.boolean(),
			css: z.boolean(),
			layers: z.boolean(),
			incremental: z
				.boolean()
				.or(z.literal("safe"))
				.or(z.literal("advance"))
				.or(z.literal("advance-silent"))
				.or(incremental),
			parallelCodeSplitting: z.boolean(),
			futureDefaults: z.boolean(),
			rspackFuture: rspackFutureOptions,
			buildHttp: buildHttpOptions,
			parallelLoader: z.boolean(),
			useInputFileSystem: useInputFileSystem,
			inlineConst: z.boolean(),
			inlineEnum: z.boolean(),
			typeReexportsPresence: z.boolean()
		})
		.partial() satisfies z.ZodType<t.Experiments>;
	//#endregion

	//#region Watch
	const watch = z.boolean() satisfies z.ZodType<t.Watch>;
	//#endregion

	//#region WatchOptions
	const watchOptions = z
		.strictObject({
			aggregateTimeout: numberOrInfinity,
			followSymlinks: z.boolean(),
			ignored: z.string().array().or(z.instanceof(RegExp)).or(z.string()),
			poll: numberOrInfinity.or(z.boolean()),
			stdin: z.boolean()
		})
		.partial() satisfies z.ZodType<t.WatchOptions>;
	//#endregion

	//#region DevServer
	const devServer = z.custom<t.DevServer>();
	//#endregion

	//#region IgnoreWarnings
	const ignoreWarnings = z
		.instanceof(RegExp)
		.or(anyFunction)
		.or(
			z.object({
				file: z.instanceof(RegExp).optional(),
				message: z.instanceof(RegExp).optional(),
				module: z.instanceof(RegExp).optional()
			})
		)
		.array() satisfies z.ZodType<t.IgnoreWarnings>;
	//#endregion

	//#region Profile
	const profile = z.boolean() satisfies z.ZodType<t.Profile>;
	//#endregion

	//#region Amd
	const amd = z
		.literal(false)
		.or(z.record(z.string(), z.any())) satisfies z.ZodType<t.Amd>;
	//#endregion

	//#region Bail
	const bail = z.boolean() satisfies z.ZodType<t.Bail>;
	//#endregion

	//#region Performance
	const performance = z
		.strictObject({
			assetFilter: anyFunction,
			hints: z.enum(["error", "warning"]).or(z.literal(false)),
			maxAssetSize: numberOrInfinity,
			maxEntrypointSize: numberOrInfinity
		})
		.partial()
		.or(z.literal(false)) satisfies z.ZodType<t.Performance>;
	//#endregion

	const rspackOptions = z
		.strictObject({
			name: name,
			dependencies: dependencies,
			extends: z.union([z.string(), z.array(z.string())]),
			entry: entry,
			output: output,
			target: target,
			mode: mode,
			experiments: experiments,
			externals: externals,
			externalsType: getExternalsTypeSchema(),
			externalsPresets: externalsPresets,
			infrastructureLogging: infrastructureLogging,
			cache: cacheOptions,
			context: context,
			devtool: devTool,
			node: node,
			loader: loader,
			ignoreWarnings: ignoreWarnings,
			watchOptions: watchOptions,
			watch: watch,
			stats: statsValue,
			snapshot: snapshotOptions,
			optimization: optimization,
			resolve: resolveOptions,
			resolveLoader: resolveOptions,
			plugins: plugins,
			devServer: devServer,
			module: moduleOptions,
			profile: profile,
			amd: amd,
			bail: bail,
			performance: performance
		})
		.partial()
		.check(externalUmdChecker) satisfies z.ZodType<t.RspackOptions>;

	return rspackOptions;
});
