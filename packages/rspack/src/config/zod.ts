import nodePath from "node:path";
import type { AssetInfo, RawFuncUseCtx } from "@rspack/binding";
import { type SyncParseReturnType, ZodIssueCode, z } from "zod";
import { Chunk } from "../Chunk";
import { ChunkGraph } from "../ChunkGraph";
import type { Compilation, PathData } from "../Compilation";
import { Module } from "../Module";
import ModuleGraph from "../ModuleGraph";
import type * as t from "./types";
import { ZodRspackCrossChecker } from "./utils";

const filenameTemplate = z.string() satisfies z.ZodType<t.FilenameTemplate>;

const filename = filenameTemplate.or(
	z
		.function()
		.args(z.custom<PathData>(), z.custom<AssetInfo>().optional())
		.returns(z.string())
) satisfies z.ZodType<t.Filename>;

//#region Name
const name = z.string() satisfies z.ZodType<t.Name>;
//#endregion

//#region Dependencies
const dependencies = z.array(name) satisfies z.ZodType<t.Dependencies>;
//#endregion

//#region Context
const context = z.string().refine(
	val => nodePath.isAbsolute(val),
	val => ({
		message: `The provided value ${JSON.stringify(val)} must be an absolute path.`
	})
) satisfies z.ZodType<t.Context>;
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

const libraryCustomUmdObject = z.strictObject({
	amd: z.string().optional(),
	commonjs: z.string().optional(),
	root: z.string().or(z.array(z.string())).optional()
}) satisfies z.ZodType<t.LibraryCustomUmdObject>;

const libraryName = z
	.string()
	.or(z.array(z.string()))
	.or(libraryCustomUmdObject) satisfies z.ZodType<t.LibraryName>;

const libraryCustomUmdCommentObject = z.strictObject({
	amd: z.string().optional(),
	commonjs: z.string().optional(),
	commonjs2: z.string().optional(),
	root: z.string().optional()
}) satisfies z.ZodType<t.LibraryCustomUmdCommentObject>;

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
	entryItem.or(entryDescription)
) satisfies z.ZodType<t.EntryObject>;

const entryStatic = entryObject.or(
	entryUnnamed
) satisfies z.ZodType<t.EntryStatic>;

const entryDynamic = z
	.function()
	.returns(
		entryStatic.or(z.promise(entryStatic))
	) satisfies z.ZodType<t.EntryDynamic>;

const entry = entryStatic.or(entryDynamic) satisfies z.ZodType<t.Entry>;
//#endregion

//#region Output
const path = z.string() satisfies z.ZodType<t.Path>;

const pathinfo = z
	.boolean()
	.or(z.literal("verbose")) satisfies z.ZodType<t.Pathinfo>;

const assetModuleFilename = filename satisfies z.ZodType<t.AssetModuleFilename>;

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

const chunkLoadingGlobal = z.string() satisfies z.ZodType<t.ChunkLoadingGlobal>;

const enabledLibraryTypes = z.array(
	libraryType
) satisfies z.ZodType<t.EnabledLibraryTypes>;

const clean = z.union([
	z.boolean(),
	z.strictObject({
		keep: z.string().optional()
	})
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

const importFunctionName = z.string() satisfies z.ZodType<t.ImportFunctionName>;

const importMetaName = z.string() satisfies z.ZodType<t.ImportMetaName>;

const iife = z.boolean() satisfies z.ZodType<t.Iife>;

const enabledChunkLoadingTypes = z.array(
	chunkLoadingType
) satisfies z.ZodType<t.EnabledChunkLoadingTypes>;

const chunkFormat = z
	.literal(false)
	.or(z.string()) satisfies z.ZodType<t.ChunkFormat>;

const workerPublicPath = z.string() satisfies z.ZodType<t.WorkerPublicPath>;

const trustedTypes = z.strictObject({
	policyName: z.string().optional(),
	onPolicyCreationFailure: z.enum(["continue", "stop"]).optional()
}) satisfies z.ZodType<t.TrustedTypes>;

const hashDigest = z.string() satisfies z.ZodType<t.HashDigest>;

const hashDigestLength = z.number() satisfies z.ZodType<t.HashDigestLength>;

const hashFunction = z.enum([
	"md4",
	"xxhash64"
]) satisfies z.ZodType<t.HashFunction>;

const hashSalt = z.string() satisfies z.ZodType<t.HashSalt>;

const sourceMapFilename = z.string() satisfies z.ZodType<t.SourceMapFilename>;

const devtoolNamespace = z.string() satisfies z.ZodType<t.DevtoolNamespace>;

const devtoolModuleFilenameTemplate = z.union([
	z.string(),
	z.function(z.tuple([z.any()]), z.any())
]) satisfies z.ZodType<t.DevtoolModuleFilenameTemplate>;

const devtoolFallbackModuleFilenameTemplate =
	devtoolModuleFilenameTemplate satisfies z.ZodType<t.DevtoolFallbackModuleFilenameTemplate>;

const environment = z.strictObject({
	arrowFunction: z.boolean().optional(),
	asyncFunction: z.boolean().optional(),
	bigIntLiteral: z.boolean().optional(),
	const: z.boolean().optional(),
	destructuring: z.boolean().optional(),
	document: z.boolean().optional(),
	dynamicImport: z.boolean().optional(),
	dynamicImportInWorker: z.boolean().optional(),
	forOf: z.boolean().optional(),
	globalThis: z.boolean().optional(),
	module: z.boolean().optional(),
	nodePrefixForCoreModules: z.boolean().optional(),
	optionalChaining: z.boolean().optional(),
	templateLiteral: z.boolean().optional()
}) satisfies z.ZodType<t.Environment>;

const output = z.strictObject({
	path: path.optional(),
	pathinfo: pathinfo.optional(),
	clean: clean.optional(),
	publicPath: publicPath.optional(),
	filename: filename.optional(),
	chunkFilename: chunkFilename.optional(),
	crossOriginLoading: crossOriginLoading.optional(),
	cssFilename: cssFilename.optional(),
	cssHeadDataCompression: z.boolean().optional(),
	cssChunkFilename: cssChunkFilename.optional(),
	hotUpdateMainFilename: hotUpdateMainFilename.optional(),
	hotUpdateChunkFilename: hotUpdateChunkFilename.optional(),
	hotUpdateGlobal: hotUpdateGlobal.optional(),
	assetModuleFilename: assetModuleFilename.optional(),
	uniqueName: uniqueName.optional(),
	chunkLoadingGlobal: chunkLoadingGlobal.optional(),
	enabledLibraryTypes: enabledLibraryTypes.optional(),
	library: library.optional(),
	libraryExport: libraryExport.optional(),
	libraryTarget: libraryType.optional(),
	umdNamedDefine: umdNamedDefine.optional(),
	auxiliaryComment: auxiliaryComment.optional(),
	module: outputModule.optional(),
	strictModuleExceptionHandling: strictModuleExceptionHandling.optional(),
	strictModuleErrorHandling: strictModuleErrorHandling.optional(),
	globalObject: globalObject.optional(),
	importFunctionName: importFunctionName.optional(),
	importMetaName: importMetaName.optional(),
	iife: iife.optional(),
	wasmLoading: wasmLoading.optional(),
	enabledWasmLoadingTypes: enabledWasmLoadingTypes.optional(),
	webassemblyModuleFilename: webassemblyModuleFilename.optional(),
	chunkFormat: chunkFormat.optional(),
	chunkLoading: chunkLoading.optional(),
	enabledChunkLoadingTypes: enabledChunkLoadingTypes.optional(),
	trustedTypes: z.literal(true).or(z.string()).or(trustedTypes).optional(),
	sourceMapFilename: sourceMapFilename.optional(),
	hashDigest: hashDigest.optional(),
	hashDigestLength: hashDigestLength.optional(),
	hashFunction: hashFunction.optional(),
	hashSalt: hashSalt.optional(),
	asyncChunks: asyncChunks.optional(),
	workerChunkLoading: chunkLoading.optional(),
	workerWasmLoading: wasmLoading.optional(),
	workerPublicPath: workerPublicPath.optional(),
	scriptType: scriptType.optional(),
	devtoolNamespace: devtoolNamespace.optional(),
	devtoolModuleFilenameTemplate: devtoolModuleFilenameTemplate.optional(),
	devtoolFallbackModuleFilenameTemplate:
		devtoolFallbackModuleFilenameTemplate.optional(),
	chunkLoadTimeout: z.number().optional(),
	charset: z.boolean().optional(),
	environment: environment.optional(),
	compareBeforeEmit: z.boolean().optional()
}) satisfies z.ZodType<t.Output>;
//#endregion

//#region Resolve
const resolveAlias = z.record(
	z
		.literal(false)
		.or(z.string())
		.or(z.array(z.string().or(z.literal(false))))
) satisfies z.ZodType<t.ResolveAlias>;

const resolveTsConfigFile = z.string();
const resolveTsConfig = resolveTsConfigFile.or(
	z.strictObject({
		configFile: resolveTsConfigFile,
		references: z.array(z.string()).or(z.literal("auto")).optional()
	})
) satisfies z.ZodType<t.ResolveTsConfig>;

const baseResolveOptions = z.strictObject({
	alias: resolveAlias.optional(),
	conditionNames: z.array(z.string()).optional(),
	extensions: z.array(z.string()).optional(),
	fallback: resolveAlias.optional(),
	mainFields: z.array(z.string()).optional(),
	mainFiles: z.array(z.string()).optional(),
	modules: z.array(z.string()).optional(),
	preferRelative: z.boolean().optional(),
	preferAbsolute: z.boolean().optional(),
	symlinks: z.boolean().optional(),
	enforceExtension: z.boolean().optional(),
	importsFields: z.array(z.string()).optional(),
	descriptionFiles: z.array(z.string()).optional(),
	tsConfig: resolveTsConfig.optional(),
	fullySpecified: z.boolean().optional(),
	exportsFields: z.array(z.string()).optional(),
	extensionAlias: z.record(z.string().or(z.array(z.string()))).optional(),
	aliasFields: z.array(z.string()).optional(),
	restrictions: z.array(z.string()).optional(),
	roots: z.array(z.string()).optional(),
	pnp: z.boolean().optional()
}) satisfies z.ZodType<t.ResolveOptions>;

const resolveOptions: z.ZodType<t.ResolveOptions> = baseResolveOptions.extend({
	byDependency: z.lazy(() => z.record(resolveOptions)).optional()
});

//#endregion

//#region Module
const baseRuleSetCondition = z
	.instanceof(RegExp)
	.or(z.string())
	.or(z.function().args(z.string()).returns(z.boolean()));

const ruleSetCondition: z.ZodType<t.RuleSetCondition> = baseRuleSetCondition
	.or(z.lazy(() => ruleSetConditions))
	.or(z.lazy(() => ruleSetLogicalConditions));

const ruleSetConditions: z.ZodType<t.RuleSetConditions> = z.lazy(() =>
	z.array(ruleSetCondition)
);

const ruleSetLogicalConditions: z.ZodType<t.RuleSetLogicalConditions> =
	z.strictObject({
		and: ruleSetConditions.optional(),
		or: ruleSetConditions.optional(),
		not: ruleSetCondition.optional()
	});

const ruleSetLoader = z.string() satisfies z.ZodType<t.RuleSetLoader>;

const ruleSetLoaderOptions = z
	.string()
	.or(z.record(z.any())) satisfies z.ZodType<t.RuleSetLoaderOptions>;

const ruleSetLoaderWithOptions = z.strictObject({
	ident: z.string().optional(),
	loader: ruleSetLoader,
	options: ruleSetLoaderOptions.optional()
}) satisfies z.ZodType<t.RuleSetLoaderWithOptions>;

const ruleSetUseItem = ruleSetLoader.or(
	ruleSetLoaderWithOptions
) satisfies z.ZodType<t.RuleSetUseItem>;

const ruleSetUse = ruleSetUseItem
	.or(ruleSetUseItem.array())
	.or(
		z.function().args(z.custom<RawFuncUseCtx>()).returns(ruleSetUseItem.array())
	) satisfies z.ZodType<t.RuleSetUse>;

const baseRuleSetRule = z.strictObject({
	test: ruleSetCondition.optional(),
	exclude: ruleSetCondition.optional(),
	include: ruleSetCondition.optional(),
	issuer: ruleSetCondition.optional(),
	issuerLayer: ruleSetCondition.optional(),
	dependency: ruleSetCondition.optional(),
	resource: ruleSetCondition.optional(),
	resourceFragment: ruleSetCondition.optional(),
	resourceQuery: ruleSetCondition.optional(),
	scheme: ruleSetCondition.optional(),
	mimetype: ruleSetCondition.optional(),
	descriptionData: z.record(ruleSetCondition).optional(),
	with: z.record(ruleSetCondition).optional(),
	type: z.string().optional(),
	layer: z.string().optional(),
	loader: ruleSetLoader.optional(),
	options: ruleSetLoaderOptions.optional(),
	use: ruleSetUse.optional(),
	parser: z.record(z.any()).optional(),
	generator: z.record(z.any()).optional(),
	resolve: resolveOptions.optional(),
	sideEffects: z.boolean().optional(),
	enforce: z.literal("pre").or(z.literal("post")).optional()
}) satisfies z.ZodType<t.RuleSetRule>;

const ruleSetRule: z.ZodType<t.RuleSetRule> = baseRuleSetRule.extend({
	oneOf: z.lazy(() => ruleSetRule.or(falsy).array()).optional(),
	rules: z.lazy(() => ruleSetRule.or(falsy).array()).optional()
});

const ruleSetRules = z.array(
	z.literal("...").or(ruleSetRule).or(falsy)
) satisfies z.ZodType<t.RuleSetRules>;

const assetParserDataUrlOptions = z.strictObject({
	maxSize: z.number().optional()
}) satisfies z.ZodType<t.AssetParserDataUrlOptions>;

const assetParserDataUrl =
	assetParserDataUrlOptions satisfies z.ZodType<t.AssetParserDataUrl>;

const assetParserOptions = z.strictObject({
	dataUrlCondition: assetParserDataUrl.optional()
}) satisfies z.ZodType<t.AssetParserOptions>;

const cssParserNamedExports =
	z.boolean() satisfies z.ZodType<t.CssParserNamedExports>;

const cssParserOptions = z.strictObject({
	namedExports: cssParserNamedExports.optional()
}) satisfies z.ZodType<t.CssParserOptions>;

const cssAutoParserOptions = z.strictObject({
	namedExports: cssParserNamedExports.optional()
}) satisfies z.ZodType<t.CssAutoParserOptions>;

const cssModuleParserOptions = z.strictObject({
	namedExports: cssParserNamedExports.optional()
}) satisfies z.ZodType<t.CssModuleParserOptions>;

const dynamicImportMode = z.enum(["eager", "lazy", "weak", "lazy-once"]);
const dynamicImportPreload = z.union([z.boolean(), z.number()]);
const dynamicImportPrefetch = z.union([z.boolean(), z.number()]);
const dynamicImportFetchPriority = z.enum(["low", "high", "auto"]);
const javascriptParserUrl = z.union([z.literal("relative"), z.boolean()]);
const exprContextCritical = z.boolean();
const wrappedContextCritical = z.boolean();
const wrappedContextRegExp = z.instanceof(RegExp);
const exportsPresence = z.enum(["error", "warn", "auto"]).or(z.literal(false));
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

const javascriptParserOptions = z.strictObject({
	dynamicImportMode: dynamicImportMode.optional(),
	dynamicImportPreload: dynamicImportPreload.optional(),
	dynamicImportPrefetch: dynamicImportPrefetch.optional(),
	dynamicImportFetchPriority: dynamicImportFetchPriority.optional(),
	importMeta: z.boolean().optional(),
	url: javascriptParserUrl.optional(),
	exprContextCritical: exprContextCritical.optional(),
	wrappedContextCritical: wrappedContextCritical.optional(),
	wrappedContextRegExp: wrappedContextRegExp.optional(),
	exportsPresence: exportsPresence.optional(),
	importExportsPresence: importExportsPresence.optional(),
	reexportExportsPresence: reexportExportsPresence.optional(),
	strictExportPresence: strictExportPresence.optional(),
	worker: worker.optional(),
	overrideStrict: overrideStrict.optional(),
	// #region Not available in webpack yet.
	requireAsExpression: requireAsExpression.optional(),
	requireDynamic: requireDynamic.optional(),
	requireResolve: requireResolve.optional(),
	importDynamic: importDynamic.optional()
	// #endregion
}) satisfies z.ZodType<t.JavascriptParserOptions>;

const parserOptionsByModuleTypeKnown = z.strictObject({
	asset: assetParserOptions.optional(),
	css: cssParserOptions.optional(),
	"css/auto": cssAutoParserOptions.optional(),
	"css/module": cssModuleParserOptions.optional(),
	javascript: javascriptParserOptions.optional(),
	"javascript/auto": javascriptParserOptions.optional(),
	"javascript/dynamic": javascriptParserOptions.optional(),
	"javascript/esm": javascriptParserOptions.optional()
}) satisfies z.ZodType<t.ParserOptionsByModuleTypeKnown>;

const parserOptionsByModuleTypeUnknown = z.record(
	z.record(z.any())
) satisfies z.ZodType<t.ParserOptionsByModuleTypeUnknown>;

const parserOptionsByModuleType = parserOptionsByModuleTypeKnown.or(
	parserOptionsByModuleTypeUnknown
) satisfies z.ZodType<t.ParserOptionsByModuleType>;

const assetGeneratorDataUrlOptions = z.strictObject({
	encoding: z.literal(false).or(z.literal("base64")).optional(),
	mimetype: z.string().optional()
}) satisfies z.ZodType<t.AssetGeneratorDataUrlOptions>;

const assetGeneratorDataUrlFunction = z
	.function()
	.args(
		z.instanceof(Buffer),
		z.strictObject({
			filename: z.string(),
			module: z.custom<Module>()
		})
	)
	.returns(z.string()) satisfies z.ZodType<t.AssetGeneratorDataUrlFunction>;

const assetGeneratorDataUrl = assetGeneratorDataUrlOptions.or(
	assetGeneratorDataUrlFunction
) satisfies z.ZodType<t.AssetGeneratorDataUrl>;

const assetInlineGeneratorOptions = z.strictObject({
	dataUrl: assetGeneratorDataUrl.optional()
}) satisfies z.ZodType<t.AssetInlineGeneratorOptions>;

const assetResourceGeneratorOptions = z.strictObject({
	emit: z.boolean().optional(),
	filename: filename.optional(),
	publicPath: publicPath.optional()
}) satisfies z.ZodType<t.AssetResourceGeneratorOptions>;

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

const cssGeneratorOptions = z.strictObject({
	exportsOnly: cssGeneratorExportsOnly.optional(),
	esModule: cssGeneratorEsModule.optional()
}) satisfies z.ZodType<t.CssGeneratorOptions>;

const cssAutoGeneratorOptions = z.strictObject({
	exportsConvention: cssGeneratorExportsConvention.optional(),
	exportsOnly: cssGeneratorExportsOnly.optional(),
	localIdentName: cssGeneratorLocalIdentName.optional(),
	esModule: cssGeneratorEsModule.optional()
}) satisfies z.ZodType<t.CssAutoGeneratorOptions>;

const cssModuleGeneratorOptions = z.strictObject({
	exportsConvention: cssGeneratorExportsConvention.optional(),
	exportsOnly: cssGeneratorExportsOnly.optional(),
	localIdentName: cssGeneratorLocalIdentName.optional(),
	esModule: cssGeneratorEsModule.optional()
}) satisfies z.ZodType<t.CssModuleGeneratorOptions>;

const jsonGeneratorOptions = z.strictObject({
	JSONParse: z.boolean().optional()
}) satisfies z.ZodType<t.JsonGeneratorOptions>;

const generatorOptionsByModuleTypeKnown = z.strictObject({
	asset: assetGeneratorOptions.optional(),
	"asset/inline": assetInlineGeneratorOptions.optional(),
	"asset/resource": assetResourceGeneratorOptions.optional(),
	css: cssGeneratorOptions.optional(),
	"css/auto": cssAutoGeneratorOptions.optional(),
	"css/module": cssModuleGeneratorOptions.optional(),
	json: jsonGeneratorOptions.optional()
}) satisfies z.ZodType<t.GeneratorOptionsByModuleTypeKnown>;

const generatorOptionsByModuleTypeUnknown = z.record(
	z.record(z.any())
) satisfies z.ZodType<t.GeneratorOptionsByModuleTypeUnknown>;

const generatorOptionsByModuleType = generatorOptionsByModuleTypeKnown.or(
	generatorOptionsByModuleTypeUnknown
) satisfies z.ZodType<t.GeneratorOptionsByModuleType>;

const noParseOptionSingle = z
	.string()
	.or(z.instanceof(RegExp))
	.or(z.function().args(z.string()).returns(z.boolean()));
const noParseOption = noParseOptionSingle.or(
	z.array(noParseOptionSingle)
) satisfies z.ZodType<t.NoParseOption>;

const moduleOptions = z.strictObject({
	defaultRules: ruleSetRules.optional(),
	rules: ruleSetRules.optional(),
	parser: parserOptionsByModuleType.optional(),
	generator: generatorOptionsByModuleType.optional(),
	noParse: noParseOption.optional()
}) satisfies z.ZodType<t.ModuleOptions>;
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
		"es2022"
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
		value => typeof value === "string" && /^electron\d+\.\d+-main$/.test(value)
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

//#region ExternalsType
export const externalsType = z.enum([
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
]) satisfies z.ZodType<t.ExternalsType>;
//#endregion

const ZodExternalObjectValue = new ZodRspackCrossChecker<
	t.ExternalItemUmdValue | t.ExternalItemObjectValue
>({
	patterns: [
		{
			test: config => {
				let isLibraryUmd = false;
				const library = config?.output?.library;
				if (typeof library === "object" && "type" in library) {
					isLibraryUmd = library.type === "umd";
				} else {
					isLibraryUmd = config?.output?.libraryTarget === "umd";
				}
				if (isLibraryUmd) {
					return (
						config?.externalsType === undefined ||
						config?.externalsType === "umd"
					);
				}
				return false;
			},
			type: z.strictObject({
				root: z.string().or(z.string().array()),
				commonjs: z.string().or(z.string().array()),
				commonjs2: z.string().or(z.string().array()),
				amd: z.string().or(z.string().array())
			}),
			issue: res => {
				if ((res as SyncParseReturnType).status === "aborted") {
					return [
						{
							fatal: true,
							code: ZodIssueCode.custom,
							message: `External object must have "root", "commonjs", "commonjs2", "amd" properties when "libraryType" or "externalsType" is "umd"`
						}
					];
				}
				return [];
			}
		}
	],
	default: z.record(z.string().or(z.string().array()))
});

// #region Externals
const externalItemValue = z
	.string()
	.or(z.boolean())
	.or(z.string().array().min(1))
	.or(ZodExternalObjectValue) satisfies z.ZodType<t.ExternalItemValue>;

const externalItemObjectUnknown = z.record(
	externalItemValue
) satisfies z.ZodType<t.ExternalItemObjectUnknown>;

const externalItemFunctionData = z.strictObject({
	context: z.string().optional(),
	dependencyType: z.string().optional(),
	request: z.string().optional(),
	contextInfo: z
		.strictObject({
			issuer: z.string(),
			issuerLayer: z.string().or(z.null()).optional()
		})
		.optional(),
	getResolve: z
		.function()
		.returns(
			z
				.function()
				.args(z.string(), z.string())
				.returns(z.promise(z.string()))
				.or(
					z
						.function()
						.args(
							z.string(),
							z.string(),
							z
								.function()
								.args(z.instanceof(Error).optional(), z.string().optional())
								.returns(z.void())
						)
						.returns(z.void())
				)
		)
		.optional()
}) satisfies z.ZodType<t.ExternalItemFunctionData>;

const externalItem = z
	.string()
	.or(z.instanceof(RegExp))
	.or(externalItemObjectUnknown)
	.or(
		z
			.function()
			.args(
				externalItemFunctionData as z.ZodType<t.ExternalItemFunctionData>,
				z
					.function()
					.args(
						z.instanceof(Error).optional(),
						externalItemValue.optional(),
						externalsType.optional()
					)
					.returns(z.void())
			)
	)
	.or(
		z
			.function()
			.args(externalItemFunctionData as z.ZodType<t.ExternalItemFunctionData>)
			.returns(z.promise(externalItemValue))
	)
	.or(
		z
			.function()
			.args(externalItemFunctionData as z.ZodType<t.ExternalItemFunctionData>)
			.returns(externalItemValue)
	) satisfies z.ZodType<t.ExternalItem>;

const externals = externalItem
	.array()
	.or(externalItem) satisfies z.ZodType<t.Externals>;

//#region ExternalsPresets
const externalsPresets = z.strictObject({
	node: z.boolean().optional(),
	web: z.boolean().optional(),
	webAsync: z.boolean().optional(),
	electron: z.boolean().optional(),
	electronMain: z.boolean().optional(),
	electronPreload: z.boolean().optional(),
	electronRenderer: z.boolean().optional(),
	nwjs: z.boolean().optional()
}) satisfies z.ZodType<t.ExternalsPresets>;
//#endregion

//#region InfrastructureLogging
const filterItemTypes = z
	.instanceof(RegExp)
	.or(z.string())
	.or(
		z.function().args(z.string()).returns(z.boolean())
	) satisfies z.ZodType<t.FilterItemTypes>;

const filterTypes = filterItemTypes
	.array()
	.or(filterItemTypes) satisfies z.ZodType<t.FilterTypes>;

const infrastructureLogging = z.strictObject({
	appendOnly: z.boolean().optional(),
	colors: z.boolean().optional(),
	console: z.custom<Console>().optional(),
	debug: z.boolean().or(filterTypes).optional(),
	level: z.enum(["none", "error", "warn", "info", "log", "verbose"]).optional(),
	stream: z.custom<NodeJS.WritableStream>().optional()
}) satisfies z.ZodType<t.InfrastructureLogging>;
//#endregion

//#region DevTool
const devTool = z
	.literal(false)
	.or(
		z.enum([
			"eval",
			"cheap-source-map",
			"cheap-module-source-map",
			"source-map",
			"inline-cheap-source-map",
			"inline-cheap-module-source-map",
			"inline-source-map",
			"inline-nosources-cheap-source-map",
			"inline-nosources-cheap-module-source-map",
			"inline-nosources-source-map",
			"nosources-cheap-source-map",
			"nosources-cheap-module-source-map",
			"nosources-source-map",
			"hidden-nosources-cheap-source-map",
			"hidden-nosources-cheap-module-source-map",
			"hidden-nosources-source-map",
			"hidden-cheap-source-map",
			"hidden-cheap-module-source-map",
			"hidden-source-map",
			"eval-cheap-source-map",
			"eval-cheap-module-source-map",
			"eval-source-map",
			"eval-nosources-cheap-source-map",
			"eval-nosources-cheap-module-source-map",
			"eval-nosources-source-map"
		])
	) satisfies z.ZodType<t.DevTool>;
//#endregion

//#region Node
const nodeOptions = z.strictObject({
	__dirname: z
		.boolean()
		.or(z.enum(["warn-mock", "mock", "eval-only", "node-module"]))
		.optional(),
	__filename: z
		.boolean()
		.or(z.enum(["warn-mock", "mock", "eval-only", "node-module"]))
		.optional(),
	global: z.boolean().or(z.literal("warn")).optional()
}) satisfies z.ZodType<t.NodeOptions>;

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
const statsOptions = z.strictObject({
	all: z.boolean().optional(),
	preset: z.boolean().or(statsPresets).optional(),
	assets: z.boolean().optional(),
	chunks: z.boolean().optional(),
	modules: z.boolean().optional(),
	entrypoints: z.boolean().or(z.literal("auto")).optional(),
	chunkGroups: z.boolean().optional(),
	warnings: z.boolean().optional(),
	warningsCount: z.boolean().optional(),
	errors: z.boolean().optional(),
	errorsCount: z.boolean().optional(),
	colors: z.boolean().optional(),
	hash: z.boolean().optional(),
	version: z.boolean().optional(),
	reasons: z.boolean().optional(),
	publicPath: z.boolean().optional(),
	outputPath: z.boolean().optional(),
	chunkModules: z.boolean().optional(),
	chunkRelations: z.boolean().optional(),
	ids: z.boolean().optional(),
	timings: z.boolean().optional(),
	builtAt: z.boolean().optional(),
	moduleAssets: z.boolean().optional(),
	nestedModules: z.boolean().optional(),
	source: z.boolean().optional(),
	logging: z
		.enum(["none", "error", "warn", "info", "log", "verbose"])
		.or(z.boolean())
		.optional(),
	loggingDebug: z.boolean().or(filterTypes).optional(),
	loggingTrace: z.boolean().optional(),
	runtimeModules: z.boolean().optional(),
	children: z.boolean().optional(),
	usedExports: z.boolean().optional(),
	providedExports: z.boolean().optional(),
	optimizationBailout: z.boolean().optional(),
	groupModulesByType: z.boolean().optional(),
	groupModulesByCacheStatus: z.boolean().optional(),
	groupModulesByLayer: z.boolean().optional(),
	groupModulesByAttributes: z.boolean().optional(),
	groupModulesByPath: z.boolean().optional(),
	groupModulesByExtension: z.boolean().optional(),
	modulesSpace: z.number().optional(),
	chunkModulesSpace: z.number().optional(),
	nestedModulesSpace: z.number().optional(),
	relatedAssets: z.boolean().optional(),
	groupAssetsByEmitStatus: z.boolean().optional(),
	groupAssetsByInfo: z.boolean().optional(),
	groupAssetsByPath: z.boolean().optional(),
	groupAssetsByExtension: z.boolean().optional(),
	groupAssetsByChunk: z.boolean().optional(),
	assetsSpace: z.number().optional(),
	orphanModules: z.boolean().optional(),
	excludeModules: z
		.array(
			z
				.string()
				.or(z.instanceof(RegExp))
				.or(z.function(z.tuple([z.string(), z.any(), z.any()]), z.boolean()))
		)
		.or(z.string())
		.or(z.instanceof(RegExp))
		.or(z.function(z.tuple([z.string(), z.any(), z.any()]), z.boolean()))
		.or(z.boolean())
		.optional(),
	excludeAssets: z
		.array(
			z
				.string()
				.or(z.instanceof(RegExp))
				.or(z.function(z.tuple([z.string(), z.any()]), z.boolean()))
		)
		.or(z.string())
		.or(z.instanceof(RegExp))
		.or(z.function(z.tuple([z.string(), z.any()]), z.boolean()))
		.optional(),
	modulesSort: z.string().optional(),
	chunkModulesSort: z.string().optional(),
	nestedModulesSort: z.string().optional(),
	chunksSort: z.string().optional(),
	assetsSort: z.string().optional(),
	performance: z.boolean().optional(),
	env: z.boolean().optional(),
	chunkGroupAuxiliary: z.boolean().optional(),
	chunkGroupChildren: z.boolean().optional(),
	chunkGroupMaxAssets: z.number().optional(),
	dependentModules: z.boolean().optional(),
	chunkOrigins: z.boolean().optional(),
	runtime: z.boolean().optional(),
	depth: z.boolean().optional(),
	reasonsSpace: z.number().optional(),
	groupReasonsByOrigin: z.boolean().optional(),
	errorDetails: z.boolean().optional(),
	errorStack: z.boolean().optional(),
	moduleTrace: z.boolean().optional(),
	cachedModules: z.boolean().optional(),
	cachedAssets: z.boolean().optional(),
	cached: z.boolean().optional(),
	errorsSpace: z.number().optional(),
	warningsSpace: z.number().optional()
}) satisfies z.ZodType<t.StatsOptions>;

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
		z.strictObject({
			name: z
				.string()
				.or(
					z
						.function()
						.args(z.strictObject({ name: z.string() }))
						.returns(z.string())
				)
				.optional()
		})
	) satisfies z.ZodType<t.OptimizationRuntimeChunk>;

const optimizationSplitChunksNameFunction = z
	.function()
	.args(z.instanceof(Module), z.array(z.instanceof(Chunk)), z.string())
	.returns(
		z.string().optional()
	) satisfies z.ZodType<t.OptimizationSplitChunksNameFunction>;

const optimizationSplitChunksName = z
	.string()
	.or(z.literal(false))
	.or(optimizationSplitChunksNameFunction);
const optimizationSplitChunksChunks = z
	.enum(["initial", "async", "all"])
	.or(z.instanceof(RegExp))
	.or(
		z
			.function()
			.args(z.instanceof(Chunk, { message: "Input not instance of Chunk" }))
			.returns(z.boolean())
	);
const optimizationSplitChunksSizes = z.number().or(z.record(z.number()));
const optimizationSplitChunksDefaultSizeTypes = z.array(z.string());
const sharedOptimizationSplitChunksCacheGroup = {
	chunks: optimizationSplitChunksChunks.optional(),
	defaultSizeTypes: optimizationSplitChunksDefaultSizeTypes.optional(),
	minChunks: z.number().min(1).optional(),
	usedExports: z.boolean().optional(),
	name: optimizationSplitChunksName.optional(),
	filename: filename.optional(),
	minSize: optimizationSplitChunksSizes.optional(),
	maxSize: optimizationSplitChunksSizes.optional(),
	maxAsyncSize: optimizationSplitChunksSizes.optional(),
	maxInitialSize: optimizationSplitChunksSizes.optional(),
	maxAsyncRequests: z.number().optional(),
	maxInitialRequests: z.number().optional(),
	automaticNameDelimiter: z.string().optional()
};
const optimizationSplitChunksCacheGroup = z.strictObject({
	test: z
		.string()
		.or(z.instanceof(RegExp))
		.or(
			z
				.function()
				.args(
					z.instanceof(Module) /** FIXME: lack of CacheGroupContext */,
					z.object({
						moduleGraph: z.instanceof(ModuleGraph),
						chunkGraph: z.instanceof(ChunkGraph)
					})
				)
				.returns(z.boolean())
		)
		.optional(),
	priority: z.number().optional(),
	enforce: z.boolean().optional(),
	reuseExistingChunk: z.boolean().optional(),
	type: z.string().or(z.instanceof(RegExp)).optional(),
	idHint: z.string().optional(),
	layer: z
		.string()
		.or(z.instanceof(RegExp))
		.or(z.function(z.tuple([z.string().optional()]), z.boolean()))
		.optional(),
	...sharedOptimizationSplitChunksCacheGroup
}) satisfies z.ZodType<t.OptimizationSplitChunksCacheGroup>;

const optimizationSplitChunksOptions = z.strictObject({
	cacheGroups: z
		.record(z.literal(false).or(optimizationSplitChunksCacheGroup))
		.optional(),
	fallbackCacheGroup: z
		.strictObject({
			chunks: optimizationSplitChunksChunks.optional(),
			minSize: z.number().optional(),
			maxSize: z.number().optional(),
			maxAsyncSize: z.number().optional(),
			maxInitialSize: z.number().optional(),
			automaticNameDelimiter: z.string().optional()
		})
		.optional(),
	hidePathInfo: z.boolean().optional(),
	...sharedOptimizationSplitChunksCacheGroup
}) satisfies z.ZodType<t.OptimizationSplitChunksOptions>;

const optimization = z.strictObject({
	moduleIds: z.enum(["named", "natural", "deterministic"]).optional(),
	chunkIds: z
		.enum(["natural", "named", "deterministic", "size", "total-size"])
		.optional(),
	minimize: z.boolean().optional(),
	minimizer: z.literal("...").or(plugin).array().optional(),
	mergeDuplicateChunks: z.boolean().optional(),
	splitChunks: z.literal(false).or(optimizationSplitChunksOptions).optional(),
	runtimeChunk: optimizationRuntimeChunk.optional(),
	removeAvailableModules: z.boolean().optional(),
	removeEmptyChunks: z.boolean().optional(),
	realContentHash: z.boolean().optional(),
	sideEffects: z.enum(["flag"]).or(z.boolean()).optional(),
	providedExports: z.boolean().optional(),
	concatenateModules: z.boolean().optional(),
	innerGraph: z.boolean().optional(),
	usedExports: z.enum(["global"]).or(z.boolean()).optional(),
	mangleExports: z.enum(["size", "deterministic"]).or(z.boolean()).optional(),
	nodeEnv: z.union([z.string(), z.literal(false)]).optional(),
	emitOnErrors: z.boolean().optional(),
	avoidEntryIife: z.boolean().optional()
}) satisfies z.ZodType<t.Optimization>;
//#endregion

//#region Experiments
const rspackFutureOptions = z.strictObject({
	bundlerInfo: z
		.strictObject({
			version: z.string().optional(),
			bundler: z.string().optional(),
			force: z
				.boolean()
				.or(z.array(z.enum(["version", "uniqueId"])))
				.optional()
		})
		.optional()
}) satisfies z.ZodType<t.RspackFutureOptions>;

const listenOptions = z.object({
	port: z.number().optional(),
	host: z.string().optional(),
	backlog: z.number().optional(),
	path: z.string().optional(),
	exclusive: z.boolean().optional(),
	readableAll: z.boolean().optional(),
	writableAll: z.boolean().optional(),
	ipv6Only: z.boolean().optional()
});

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

const lazyCompilationOptions = z.object({
	backend: z
		.object({
			client: z.string().optional(),
			listen: z
				.number()
				.or(listenOptions)
				.or(z.function().args(z.any()).returns(z.void()))
				.optional(),
			protocol: z.enum(["http", "https"]).optional(),
			server: z.record(z.any()).or(z.function().returns(z.any())).optional()
		})
		.optional(),
	imports: z.boolean().optional(),
	entries: z.boolean().optional(),
	test: z
		.instanceof(RegExp)
		.or(z.function().args(z.custom<Module>()).returns(z.boolean()))
		.optional()
}) satisfies z.ZodType<t.LazyCompilationOptions>;

const incremental = z.strictObject({
	make: z.boolean().optional(),
	inferAsyncModules: z.boolean().optional(),
	providedExports: z.boolean().optional(),
	dependenciesDiagnostics: z.boolean().optional(),
	sideEffects: z.boolean().optional(),
	buildChunkGraph: z.boolean().optional(),
	moduleIds: z.boolean().optional(),
	chunkIds: z.boolean().optional(),
	modulesHashes: z.boolean().optional(),
	modulesCodegen: z.boolean().optional(),
	modulesRuntimeRequirements: z.boolean().optional(),
	chunksRuntimeRequirements: z.boolean().optional(),
	chunksHashes: z.boolean().optional(),
	chunksRender: z.boolean().optional(),
	emitAssets: z.boolean().optional()
}) satisfies z.ZodType<t.Incremental>;

const experiments = z.strictObject({
	cache: z.boolean().optional().or(experimentCacheOptions),
	lazyCompilation: z.boolean().optional().or(lazyCompilationOptions),
	asyncWebAssembly: z.boolean().optional(),
	outputModule: z.boolean().optional(),
	topLevelAwait: z.boolean().optional(),
	css: z.boolean().optional(),
	layers: z.boolean().optional(),
	incremental: z.boolean().or(incremental).optional(),
	parallelCodeSplitting: z.boolean().optional(),
	futureDefaults: z.boolean().optional(),
	rspackFuture: rspackFutureOptions.optional()
}) satisfies z.ZodType<t.Experiments>;
//#endregion

//#region Watch
const watch = z.boolean() satisfies z.ZodType<t.Watch>;
//#endregion

//#region WatchOptions
const watchOptions = z.strictObject({
	aggregateTimeout: z.number().optional(),
	followSymlinks: z.boolean().optional(),
	ignored: z
		.string()
		.array()
		.or(z.instanceof(RegExp))
		.or(z.string())
		.optional(),
	poll: z.number().or(z.boolean()).optional(),
	stdin: z.boolean().optional()
}) satisfies z.ZodType<t.WatchOptions>;
//#endregion

//#region DevServer
const devServer = z.custom<t.DevServer>();
//#endregion

//#region IgnoreWarnings
const ignoreWarnings = z
	.instanceof(RegExp)
	.or(
		z
			.function()
			.args(z.instanceof(Error), z.custom<Compilation>())
			.returns(z.boolean())
	)
	.array() satisfies z.ZodType<t.IgnoreWarnings>;
//#endregion

//#region Profile
const profile = z.boolean() satisfies z.ZodType<t.Profile>;
//#endregion

//#region Amd
const amd = z.literal(false).or(z.record(z.any())) satisfies z.ZodType<t.Amd>;
//#endregion

//#region Bail
const bail = z.boolean() satisfies z.ZodType<t.Bail>;
//#endregion

//#region Performance
const performance = z
	.strictObject({
		assetFilter: z.function().args(z.string()).returns(z.boolean()).optional(),
		hints: z.enum(["error", "warning"]).or(z.literal(false)).optional(),
		maxAssetSize: z.number().optional(),
		maxEntrypointSize: z.number().optional()
	})
	.or(z.literal(false)) satisfies z.ZodType<t.Performance>;
//#endregion

export const rspackOptions = z.strictObject({
	name: name.optional(),
	dependencies: dependencies.optional(),
	extends: z.union([z.string(), z.array(z.string())]).optional(),
	entry: entry.optional(),
	output: output.optional(),
	target: target.optional(),
	mode: mode.optional(),
	experiments: experiments.optional(),
	externals: externals.optional(),
	externalsType: externalsType.optional(),
	externalsPresets: externalsPresets.optional(),
	infrastructureLogging: infrastructureLogging.optional(),
	cache: cacheOptions.optional(),
	context: context.optional(),
	devtool: devTool.optional(),
	node: node.optional(),
	loader: loader.optional(),
	ignoreWarnings: ignoreWarnings.optional(),
	watchOptions: watchOptions.optional(),
	watch: watch.optional(),
	stats: statsValue.optional(),
	snapshot: snapshotOptions.optional(),
	optimization: optimization.optional(),
	resolve: resolveOptions.optional(),
	resolveLoader: resolveOptions.optional(),
	plugins: plugins.optional(),
	devServer: devServer.optional(),
	module: moduleOptions.optional(),
	profile: profile.optional(),
	amd: amd.optional(),
	bail: bail.optional(),
	performance: performance.optional()
}) satisfies z.ZodType<t.RspackOptions>;
