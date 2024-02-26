import { RawFuncUseCtx } from "@rspack/binding";
import { z } from "zod";
import { Compilation, Compiler } from "..";
import type * as oldBuiltins from "../builtin-plugin";
import type * as webpackDevServer from "webpack-dev-server";
import { deprecatedWarn, termlink } from "../util";
import { Module } from "../Module";
import { Chunk } from "../Chunk";

//#region Name
const name = z.string();
export type Name = z.infer<typeof name>;
//#endregion

//#region Dependencies
const dependencies = z.array(name);
export type Dependencies = z.infer<typeof dependencies>;
//#endregion

//#region Context
const context = z.string();
export type Context = z.infer<typeof context>;
//#endregion

//#region Mode
const mode = z.enum(["development", "production", "none"]);
export type Mode = z.infer<typeof mode>;
//#endregion

//#region Falsy
const falsy = z.union([
	z.literal(false),
	z.literal(0),
	z.literal(""),
	z.null(),
	z.undefined()
]);

export type Falsy = z.infer<typeof falsy>;
//#endregion

//#region Entry
const rawPublicPath = z.string();
export type RawPublicPath = z.infer<typeof rawPublicPath>;

const publicPath = z.literal("auto").or(rawPublicPath);
export type PublicPath = z.infer<typeof publicPath>;

const baseUri = z.string();
export type BaseUri = z.infer<typeof baseUri>;

const chunkLoadingType = z
	.enum(["jsonp", "import-scripts", "require", "async-node", "import"])
	.or(z.string());
export type ChunkLoadingType = z.infer<typeof chunkLoadingType>;

const chunkLoading = z.literal(false).or(chunkLoadingType);
export type ChunkLoading = z.infer<typeof chunkLoading>;

const asyncChunks = z.boolean();
export type AsyncChunks = z.infer<typeof asyncChunks>;

const wasmLoadingType = z
	.enum(["fetch-streaming", "fetch", "async-node"])
	.or(z.string());
export type WasmLoadingType = z.infer<typeof wasmLoadingType>;

const wasmLoading = z.literal(false).or(wasmLoadingType);
export type WasmLoading = z.infer<typeof wasmLoading>;

const scriptType = z.enum(["text/javascript", "module"]).or(z.literal(false));
export type ScriptType = z.infer<typeof scriptType>;

const libraryCustomUmdObject = z.strictObject({
	amd: z.string().optional(),
	commonjs: z.string().optional(),
	root: z.string().or(z.array(z.string())).optional()
});
export type LibraryCustomUmdObject = z.infer<typeof libraryCustomUmdObject>;

const libraryName = z
	.string()
	.or(z.array(z.string()))
	.or(libraryCustomUmdObject);
export type LibraryName = z.infer<typeof libraryName>;

const libraryCustomUmdCommentObject = z.strictObject({
	amd: z.string().optional(),
	commonjs: z.string().optional(),
	commonjs2: z.string().optional(),
	root: z.string().optional()
});
export type LibraryCustomUmdCommentObject = z.infer<
	typeof libraryCustomUmdCommentObject
>;

const amdContainer = z.string();
export type AmdContainer = z.infer<typeof amdContainer>;

const auxiliaryComment = z.string().or(libraryCustomUmdCommentObject);
export type AuxiliaryComment = z.infer<typeof auxiliaryComment>;

const libraryExport = z.string().or(z.array(z.string()));
export type LibraryExport = z.infer<typeof libraryExport>;

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
	.or(z.string());
export type LibraryType = z.infer<typeof libraryType>;

const umdNamedDefine = z.boolean();
export type UmdNamedDefine = z.infer<typeof umdNamedDefine>;

const libraryOptions = z.strictObject({
	amdContainer: amdContainer.optional(),
	auxiliaryComment: auxiliaryComment.optional(),
	export: libraryExport.optional(),
	name: libraryName.optional(),
	type: libraryType,
	umdNamedDefine: umdNamedDefine.optional()
});
export type LibraryOptions = z.infer<typeof libraryOptions>;

const library = libraryName.or(libraryOptions).optional();
export type Library = z.infer<typeof library>;

const filenameTemplate = z.string();
export type FilenameTemplate = z.infer<typeof filenameTemplate>;

const filename = filenameTemplate;
export type Filename = z.infer<typeof filename>;

const entryFilename = filenameTemplate;
export type EntryFilename = z.infer<typeof entryFilename>;

const entryRuntime = z.literal(false).or(z.string());
export type EntryRuntime = z.infer<typeof entryRuntime>;

const entryItem = z.string().or(z.array(z.string()));
export type EntryItem = z.infer<typeof entryItem>;

const entryDescription = z.strictObject({
	import: entryItem,
	runtime: entryRuntime.optional(),
	publicPath: publicPath.optional(),
	baseUri: baseUri.optional(),
	chunkLoading: chunkLoading.optional(),
	asyncChunks: asyncChunks.optional(),
	wasmLoading: wasmLoading.optional(),
	filename: entryFilename.optional(),
	library: libraryOptions.optional()
});
export type EntryDescription = z.infer<typeof entryDescription>;

const entryUnnamed = entryItem;
export type EntryUnnamed = z.infer<typeof entryUnnamed>;

const entryObject = z.record(entryItem.or(entryDescription));
export type EntryObject = z.infer<typeof entryObject>;

const entryStatic = entryObject.or(entryUnnamed);
export type EntryStatic = z.infer<typeof entryStatic>;

const entry = entryStatic;
export type Entry = z.infer<typeof entry>;
//#endregion

//#region Output
const path = z.string();
export type Path = z.infer<typeof path>;

const assetModuleFilename = z.string();
export type AssetModuleFilename = z.infer<typeof assetModuleFilename>;

const webassemblyModuleFilename = z.string();
export type WebassemblyModuleFilename = z.infer<
	typeof webassemblyModuleFilename
>;

const chunkFilename = filenameTemplate;
export type ChunkFilename = z.infer<typeof chunkFilename>;

const crossOriginLoading = z
	.literal(false)
	.or(z.enum(["anonymous", "use-credentials"]));
export type CrossOriginLoading = z.infer<typeof crossOriginLoading>;

const cssFilename = filenameTemplate;
export type CssFilename = z.infer<typeof cssFilename>;

const cssChunkFilename = filenameTemplate;
export type CssChunkFilename = z.infer<typeof cssChunkFilename>;

const hotUpdateChunkFilename = filenameTemplate;
export type HotUpdateChunkFilename = z.infer<typeof hotUpdateChunkFilename>;

const hotUpdateMainFilename = filenameTemplate;
export type HotUpdateMainFilename = z.infer<typeof hotUpdateMainFilename>;

const hotUpdateGlobal = z.string();
export type HotUpdateGlobal = z.infer<typeof hotUpdateGlobal>;

const uniqueName = z.string();
export type UniqueName = z.infer<typeof uniqueName>;

const chunkLoadingGlobal = z.string();
export type ChunkLoadingGlobal = z.infer<typeof chunkLoadingGlobal>;

const enabledLibraryTypes = z.array(libraryType);
export type EnabledLibraryTypes = z.infer<typeof enabledLibraryTypes>;

const clean = z.boolean();
export type Clean = z.infer<typeof clean>;

const outputModule = z.boolean();
export type OutputModule = z.infer<typeof outputModule>;

const strictModuleExceptionHandling = z.boolean();
export type StrictModuleExceptionHandling = z.infer<
	typeof strictModuleExceptionHandling
>;

const strictModuleErrorHandling = z.boolean();
export type StrictModuleErrorHandling = z.infer<
	typeof strictModuleErrorHandling
>;

const globalObject = z.string();
export type GlobalObject = z.infer<typeof globalObject>;

const enabledWasmLoadingTypes = z.array(wasmLoadingType);
export type EnabledWasmLoadingTypes = z.infer<typeof enabledWasmLoadingTypes>;

const importFunctionName = z.string();
export type ImportFunctionName = z.infer<typeof importFunctionName>;

const iife = z.boolean();
export type Iife = z.infer<typeof iife>;

const enabledChunkLoadingTypes = z.array(chunkLoadingType);
export type EnabledChunkLoadingTypes = z.infer<typeof enabledChunkLoadingTypes>;

const chunkFormat = z.literal(false).or(z.string());
export type ChunkFormat = z.infer<typeof chunkFormat>;

const workerPublicPath = z.string();
export type WorkerPublicPath = z.infer<typeof workerPublicPath>;

const trustedTypes = z.strictObject({
	policyName: z.string().optional()
});
export type TrustedTypes = z.infer<typeof trustedTypes>;

const hashDigest = z.string();
export type HashDigest = z.infer<typeof hashDigest>;

const hashDigestLength = z.number();
export type HashDigestLength = z.infer<typeof hashDigestLength>;

const hashFunction = z.enum(["md4", "xxhash64"]);
export type HashFunction = z.infer<typeof hashFunction>;

const hashSalt = z.string();
export type HashSalt = z.infer<typeof hashSalt>;

const sourceMapFilename = z.string();
export type SourceMapFilename = z.infer<typeof sourceMapFilename>;

const devtoolNamespace = z.string();
export type DevtoolNamespace = z.infer<typeof devtoolNamespace>;

const devtoolModuleFilenameTemplate = z.union([
	z.string(),
	z.function(z.tuple([z.any()]), z.any())
]);
export type DevtoolModuleFilenameTemplate = z.infer<
	typeof devtoolModuleFilenameTemplate
>;

const devtoolFallbackModuleFilenameTemplate = devtoolModuleFilenameTemplate;
export type DevtoolFallbackModuleFilenameTemplate = z.infer<
	typeof devtoolFallbackModuleFilenameTemplate
>;

const output = z.strictObject({
	path: path.optional(),
	clean: clean.optional(),
	publicPath: publicPath.optional(),
	filename: filename.optional(),
	chunkFilename: chunkFilename.optional(),
	crossOriginLoading: crossOriginLoading.optional(),
	cssFilename: cssFilename.optional(),
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
	amdContainer: amdContainer.optional(),
	auxiliaryComment: auxiliaryComment.optional(),
	module: outputModule.optional(),
	strictModuleExceptionHandling: strictModuleExceptionHandling.optional(),
	strictModuleErrorHandling: strictModuleErrorHandling.optional(),
	globalObject: globalObject.optional(),
	importFunctionName: importFunctionName.optional(),
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
		devtoolFallbackModuleFilenameTemplate.optional()
});
export type Output = z.infer<typeof output>;
//#endregion

//#region Resolve
const resolveAlias = z.record(
	z
		.literal(false)
		.or(z.string())
		.or(z.array(z.string().or(z.literal(false))))
);
export type ResolveAlias = z.infer<typeof resolveAlias>;

const resolveTsconfig = z.strictObject({
	configFile: z.string(),
	references: z.array(z.string()).or(z.literal("auto")).optional()
});

export type ResolveTsconfig = z.infer<typeof resolveTsconfig>;

const baseResolveOptions = z.strictObject({
	alias: resolveAlias.optional(),
	/**
	 * This is `aliasField: ["browser"]` in webpack, because no one
	 * uses aliasField other than "browser". ---@bvanjoi
	 */
	browserField: z
		.boolean()
		.optional()
		.refine(val => {
			if (val !== undefined) {
				deprecatedWarn(
					`'resolve.browserField' has been deprecated, and will be removed in 0.6.0. Please use 'resolve.aliasField' instead.`
				);
			}
			return true;
		}),
	conditionNames: z.array(z.string()).optional(),
	extensions: z.array(z.string()).optional(),
	fallback: resolveAlias.optional(),
	mainFields: z.array(z.string()).optional(),
	mainFiles: z.array(z.string()).optional(),
	modules: z.array(z.string()).optional(),
	preferRelative: z.boolean().optional(),
	preferAbsolute: z.boolean().optional(),
	symlinks: z.boolean().optional(),
	tsConfigPath: z.string().optional(),
	tsConfig: resolveTsconfig.optional(),
	fullySpecified: z.boolean().optional(),
	exportsFields: z.array(z.string()).optional(),
	extensionAlias: z.record(z.string().or(z.array(z.string()))).optional(),
	aliasFields: z.array(z.string()).optional(),
	restrictions: z.array(z.string()).optional(),
	roots: z.array(z.string()).optional()
});

export type ResolveOptions = z.infer<typeof baseResolveOptions> & {
	byDependency?: Record<string, ResolveOptions>;
};
const resolveOptions: z.ZodType<ResolveOptions> = baseResolveOptions.extend({
	byDependency: z.lazy(() => z.record(resolveOptions)).optional()
});

const resolve = resolveOptions;
export type Resolve = z.infer<typeof resolve>;
//#endregion

//#region Module
const baseRuleSetCondition = z
	.instanceof(RegExp)
	.or(z.string())
	.or(z.function().args(z.string()).returns(z.boolean()));

export type RuleSetCondition =
	| z.infer<typeof baseRuleSetCondition>
	| RuleSetConditions
	| RuleSetLogicalConditions;

const ruleSetCondition: z.ZodType<RuleSetCondition> = baseRuleSetCondition
	.or(z.lazy(() => ruleSetConditions))
	.or(z.lazy(() => ruleSetLogicalConditions));

export type RuleSetConditions = RuleSetCondition[];

const ruleSetConditions: z.ZodType<RuleSetConditions> = z.lazy(() =>
	z.array(ruleSetCondition)
);

export type RuleSetLogicalConditions = {
	and?: RuleSetConditions;
	or?: RuleSetConditions;
	not?: RuleSetCondition;
};

const ruleSetLogicalConditions: z.ZodType<RuleSetLogicalConditions> =
	z.strictObject({
		and: ruleSetConditions.optional(),
		or: ruleSetConditions.optional(),
		not: ruleSetCondition.optional()
	});

const ruleSetLoader = z.string();
export type RuleSetLoader = z.infer<typeof ruleSetLoader>;

const ruleSetLoaderOptions = z.string().or(z.record(z.any()));
export type RuleSetLoaderOptions = z.infer<typeof ruleSetLoaderOptions>;

const ruleSetLoaderWithOptions = z.strictObject({
	ident: z.string().optional(),
	loader: ruleSetLoader,
	options: ruleSetLoaderOptions.optional()
});
export type RuleSetLoaderWithOptions = z.infer<typeof ruleSetLoaderWithOptions>;

const ruleSetUseItem = ruleSetLoader.or(ruleSetLoaderWithOptions);
export type RuleSetUseItem = z.infer<typeof ruleSetUseItem>;

const ruleSetUse = ruleSetUseItem
	.or(ruleSetUseItem.array())
	.or(
		z.function().args(z.custom<RawFuncUseCtx>()).returns(ruleSetUseItem.array())
	);
export type RuleSetUse = z.infer<typeof ruleSetUse>;

const baseRuleSetRule = z.strictObject({
	test: ruleSetCondition.optional(),
	exclude: ruleSetCondition.optional(),
	include: ruleSetCondition.optional(),
	issuer: ruleSetCondition.optional(),
	dependency: ruleSetCondition.optional(),
	resource: ruleSetCondition.optional(),
	resourceFragment: ruleSetCondition.optional(),
	resourceQuery: ruleSetCondition.optional(),
	scheme: ruleSetCondition.optional(),
	mimetype: ruleSetCondition.optional(),
	descriptionData: z.record(ruleSetCondition).optional(),
	type: z.string().optional(),
	loader: ruleSetLoader.optional(),
	options: ruleSetLoaderOptions.optional(),
	use: ruleSetUse.optional(),
	parser: z.record(z.any()).optional(),
	generator: z.record(z.any()).optional(),
	resolve: resolveOptions.optional(),
	sideEffects: z.boolean().optional(),
	enforce: z.literal("pre").or(z.literal("post")).optional()
});

export type RuleSetRule = z.infer<typeof baseRuleSetRule> & {
	oneOf?: RuleSetRule[];
	rules?: RuleSetRule[];
};

const ruleSetRule: z.ZodType<RuleSetRule> = baseRuleSetRule.extend({
	oneOf: z.lazy(() => ruleSetRule.array()).optional(),
	rules: z.lazy(() => ruleSetRule.array()).optional()
});

const ruleSetRules = z.array(z.literal("...").or(ruleSetRule).or(falsy));
export type RuleSetRules = z.infer<typeof ruleSetRules>;

const assetParserDataUrlOptions = z.strictObject({
	maxSize: z.number().optional()
});
export type AssetParserDataUrlOptions = z.infer<
	typeof assetParserDataUrlOptions
>;

const assetParserDataUrl = assetParserDataUrlOptions;
export type AssetParserDataUrl = z.infer<typeof assetParserDataUrl>;

const assetParserOptions = z.strictObject({
	dataUrlCondition: assetParserDataUrl.optional()
});
export type AssetParserOptions = z.infer<typeof assetParserOptions>;

//TODO: "weak", "lazy-once"
const dynamicImportMode = z.enum(["eager", "lazy"]);
const dynamicImportPreload = z.union([z.boolean(), z.number()]);
const dynamicImportPrefetch = z.union([z.boolean(), z.number()]);
const javascriptParserUrl = z.union([z.literal("relative"), z.boolean()]);

const javascriptParserOptions = z.strictObject({
	dynamicImportMode: dynamicImportMode.optional(),
	dynamicImportPreload: dynamicImportPreload.optional(),
	dynamicImportPrefetch: dynamicImportPrefetch.optional(),
	url: javascriptParserUrl.optional()
});
export type JavascriptParserOptions = z.infer<typeof javascriptParserOptions>;

const parserOptionsByModuleTypeKnown = z.strictObject({
	asset: assetParserOptions.optional(),
	javascript: javascriptParserOptions.optional()
});

export type ParserOptionsByModuleTypeKnown = z.infer<
	typeof parserOptionsByModuleTypeKnown
>;

const parserOptionsByModuleTypeUnknown = z.record(z.record(z.any()));
export type ParserOptionsByModuleTypeUnknown = z.infer<
	typeof parserOptionsByModuleTypeUnknown
>;

const parserOptionsByModuleType = parserOptionsByModuleTypeKnown.or(
	parserOptionsByModuleTypeUnknown
);
export type ParserOptionsByModuleType = z.infer<
	typeof parserOptionsByModuleType
>;

const assetGeneratorDataUrlOptions = z.strictObject({
	encoding: z.literal(false).or(z.literal("base64")).optional(),
	mimetype: z.string().optional()
});
export type AssetGeneratorDataUrlOptions = z.infer<
	typeof assetGeneratorDataUrlOptions
>;

const assetGeneratorDataUrl = assetGeneratorDataUrlOptions;
export type AssetGeneratorDataUrl = z.infer<typeof assetGeneratorDataUrl>;

const assetInlineGeneratorOptions = z.strictObject({
	dataUrl: assetGeneratorDataUrl.optional()
});
export type AssetInlineGeneratorOptions = z.infer<
	typeof assetInlineGeneratorOptions
>;

const assetResourceGeneratorOptions = z.strictObject({
	filename: filenameTemplate.optional(),
	publicPath: publicPath.optional()
});
export type AssetResourceGeneratorOptions = z.infer<
	typeof assetResourceGeneratorOptions
>;

const assetGeneratorOptions = assetInlineGeneratorOptions.merge(
	assetResourceGeneratorOptions
);
export type AssetGeneratorOptions = z.infer<typeof assetGeneratorOptions>;

const generatorOptionsByModuleTypeKnown = z.strictObject({
	asset: assetGeneratorOptions.optional(),
	"asset/inline": assetInlineGeneratorOptions.optional(),
	"asset/resource": assetResourceGeneratorOptions.optional()
});
export type GeneratorOptionsByModuleTypeKnown = z.infer<
	typeof generatorOptionsByModuleTypeKnown
>;

const generatorOptionsByModuleTypeUnknown = z.record(z.record(z.any()));
export type GeneratorOptionsByModuleTypeUnknown = z.infer<
	typeof generatorOptionsByModuleTypeUnknown
>;

const generatorOptionsByModuleType = generatorOptionsByModuleTypeKnown.or(
	generatorOptionsByModuleTypeUnknown
);
export type GeneratorOptionsByModuleType = z.infer<
	typeof generatorOptionsByModuleType
>;

const moduleOptions = z.strictObject({
	defaultRules: ruleSetRules.optional(),
	rules: ruleSetRules.optional(),
	parser: parserOptionsByModuleType.optional(),
	generator: generatorOptionsByModuleType.optional()
});
export type ModuleOptions = z.infer<typeof moduleOptions>;
//#endregion

//#region Target
const allowTarget = z
	.enum([
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
		"browserslist"
	])
	.or(z.literal("node"))
	.or(z.literal("async-node"))
	.or(
		z.custom<`node${number}`>(
			value => typeof value === "string" && /^node\d+$/.test(value)
		)
	)
	.or(
		z.custom<`async-node${number}`>(
			value => typeof value === "string" && /^async-node\d+$/.test(value)
		)
	)
	.or(
		z.custom<`node${number}.${number}`>(
			value => typeof value === "string" && /^node\d+\.\d+$/.test(value)
		)
	)
	.or(
		z.custom<`async-node${number}.${number}`>(
			value => typeof value === "string" && /^async-node\d+\.\d+$/.test(value)
		)
	)
	.or(z.literal("electron-main"))
	.or(
		z.custom<`electron${number}-main`>(
			value => typeof value === "string" && /^electron\d+-main$/.test(value)
		)
	)
	.or(
		z.custom<`electron${number}.${number}-main`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-main$/.test(value)
		)
	)
	.or(z.literal("electron-renderer"))
	.or(
		z.custom<`electron${number}-renderer`>(
			value => typeof value === "string" && /^electron\d+-renderer$/.test(value)
		)
	)
	.or(
		z.custom<`electron${number}.${number}-renderer`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-renderer$/.test(value)
		)
	)
	.or(z.literal("electron-preload"))
	.or(
		z.custom<`electron${number}-preload`>(
			value => typeof value === "string" && /^electron\d+-preload$/.test(value)
		)
	)
	.or(
		z.custom<`electron${number}.${number}-preload`>(
			value =>
				typeof value === "string" && /^electron\d+\.\d+-preload$/.test(value)
		)
	);
const target = z.literal(false).or(allowTarget).or(allowTarget.array());
export type Target = z.infer<typeof target>;
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
	"script",
	"node-commonjs"
]);
export type ExternalsType = z.infer<typeof externalsType>;
//#endregion

//#region Externals
const externalItemValue = z
	.string()
	.or(z.boolean())
	.or(z.string().array().min(1))
	.or(z.record(z.string().or(z.string().array())));
export type ExternalItemValue = z.infer<typeof externalItemValue>;

const externalItemObjectUnknown = z.record(externalItemValue);
export type ExternalItemObjectUnknown = z.infer<
	typeof externalItemObjectUnknown
>;

const externalItemFunctionData = z.strictObject({
	context: z.string().optional(),
	dependencyType: z.string().optional(),
	request: z.string().optional()
});
export type ExternalItemFunctionData = z.infer<typeof externalItemFunctionData>;

const externalItem = z
	.string()
	.or(z.instanceof(RegExp))
	.or(externalItemObjectUnknown)
	.or(
		z
			.function()
			.args(
				externalItemFunctionData,
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
			.args(externalItemFunctionData)
			.returns(z.promise(externalItemValue))
	);
export type ExternalItem = z.infer<typeof externalItem>;

const externals = externalItem.array().or(externalItem);
export type Externals = z.infer<typeof externals>;
//#endregion

//#region ExternalsPresets
const externalsPresets = z.strictObject({
	node: z.boolean().optional(),
	web: z.boolean().optional(),
	webAsync: z.boolean().optional(),
	electron: z.boolean().optional(),
	electronMain: z.boolean().optional(),
	electronPreload: z.boolean().optional(),
	electronRenderer: z.boolean().optional()
});
export type ExternalsPresets = z.infer<typeof externalsPresets>;
//#endregion

//#region InfrastructureLogging
const filterItemTypes = z
	.instanceof(RegExp)
	.or(z.string())
	.or(z.function().args(z.string()).returns(z.boolean()));
export type FilterItemTypes = z.infer<typeof filterItemTypes>;

const filterTypes = filterItemTypes.array().or(filterItemTypes);
export type FilterTypes = z.infer<typeof filterTypes>;

const infrastructureLogging = z.strictObject({
	appendOnly: z.boolean().optional(),
	colors: z.boolean().optional(),
	console: z.custom<Console>().optional(),
	debug: z.boolean().or(filterTypes).optional(),
	level: z.enum(["none", "error", "warn", "info", "log", "verbose"]).optional(),
	stream: z.custom<NodeJS.WritableStream>().optional()
});
export type InfrastructureLogging = z.infer<typeof infrastructureLogging>;
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
	);
export type DevTool = z.infer<typeof devTool>;
//#endregion

//#region Node
const nodeOptions = z.strictObject({
	__dirname: z
		.boolean()
		.or(z.enum(["warn-mock", "mock", "eval-only"]))
		.optional(),
	__filename: z
		.boolean()
		.or(z.enum(["warn-mock", "mock", "eval-only"]))
		.optional(),
	global: z.boolean().or(z.literal("warn")).optional()
});
export type NodeOptions = z.infer<typeof nodeOptions>;

const node = z.literal(false).or(nodeOptions);
export type Node = z.infer<typeof node>;
//#endregion

//#region Snapshot
const snapshotOptions = z.strictObject({
	module: z
		.strictObject({
			hash: z.boolean().optional(),
			timestamp: z.boolean().optional()
		})
		.optional(),
	resolve: z
		.strictObject({
			hash: z.boolean().optional(),
			timestamp: z.boolean().optional()
		})
		.optional()
});
export type SnapshotOptions = z.infer<typeof snapshotOptions>;
//#endregion

//#region Cache
const cacheOptions = z.boolean();
export type CacheOptions = z.infer<typeof cacheOptions>;
//#endregion

//#region Stats
const statsOptions = z.strictObject({
	all: z.boolean().optional(),
	preset: z
		.enum(["normal", "none", "verbose", "errors-only", "errors-warnings"])
		.optional(),
	assets: z.boolean().optional(),
	chunks: z.boolean().optional(),
	modules: z.boolean().optional(),
	entrypoints: z.boolean().optional(),
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
	modulesSpace: z.number().optional(),
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
	orphanModules: z.boolean().optional()
});
export type StatsOptions = z.infer<typeof statsOptions>;

const statsValue = z
	.enum(["none", "errors-only", "errors-warnings", "normal", "verbose"])
	.or(z.boolean())
	.or(statsOptions);
export type StatsValue = z.infer<typeof statsValue>;
//#endregion

//#region Plugins
export interface RspackPluginInstance {
	apply: (compiler: Compiler) => void;
	[k: string]: any;
}
export type RspackPluginFunction = (this: Compiler, compiler: Compiler) => void;

const plugin = z.union([
	z.custom<RspackPluginInstance>(),
	z.custom<RspackPluginFunction>(),
	falsy
]);
const plugins = plugin.array();
export type Plugins = z.infer<typeof plugins>;
//#endregion

//#region Optimization
const optimizationRuntimeChunk = z
	.enum(["single", "multiple"])
	.or(z.boolean())
	.or(
		z.strictObject({
			name: z
				.string()
				.or(z.function().returns(z.string().or(z.undefined())))
				.optional()
		})
	);
export type OptimizationRuntimeChunk = z.infer<typeof optimizationRuntimeChunk>;

const optimizationSplitChunksNameFunction = z.function().args(
	z.instanceof(Module).optional()
	// FIXME: z.array(z.instanceof(Chunk)).optional(), z.string()
	// FIXME: Chunk[],   															cacheChunkKey
);

export type OptimizationSplitChunksNameFunction = z.infer<
	typeof optimizationSplitChunksNameFunction
>;

const optimizationSplitChunksName = z
	.string()
	.or(z.literal(false))
	.or(optimizationSplitChunksNameFunction);
const optimizationSplitChunksChunks = z
	.enum(["initial", "async", "all"])
	.or(z.instanceof(RegExp))
	.or(z.function().args(z.instanceof(Chunk)).returns(z.boolean()));
const optimizationSplitChunksSizes = z.number();
const optimizationSplitChunksDefaultSizeTypes = z.array(z.string());
const sharedOptimizationSplitChunksCacheGroup = {
	chunks: optimizationSplitChunksChunks.optional(),
	defaultSizeTypes: optimizationSplitChunksDefaultSizeTypes.optional(),
	minChunks: z.number().min(1).optional(),
	name: optimizationSplitChunksName.optional(),
	minSize: optimizationSplitChunksSizes.optional(),
	maxSize: optimizationSplitChunksSizes.optional(),
	maxAsyncSize: optimizationSplitChunksSizes.optional(),
	maxInitialSize: optimizationSplitChunksSizes.optional(),
	automaticNameDelimiter: z.string().optional()
};
const optimizationSplitChunksCacheGroup = z.strictObject({
	test: z
		.string()
		.or(z.instanceof(RegExp))
		.or(
			z
				.function()
				.args(z.instanceof(Module) /** FIXME: lack of CacheGroupContext */)
		)
		.optional(),
	priority: z.number().optional(),
	enforce: z.boolean().optional(),
	filename: z.string().optional(),
	reuseExistingChunk: z.boolean().optional(),
	type: z.string().or(z.instanceof(RegExp)).optional(),
	idHint: z.string().optional(),
	...sharedOptimizationSplitChunksCacheGroup
});
export type OptimizationSplitChunksCacheGroup = z.infer<
	typeof optimizationSplitChunksCacheGroup
>;

const optimizationSplitChunksOptions = z.strictObject({
	cacheGroups: z
		.record(z.literal(false).or(optimizationSplitChunksCacheGroup))
		.optional(),
	maxAsyncRequests: z.number().optional(),
	maxInitialRequests: z.number().optional(),
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
});
export type OptimizationSplitChunksOptions = z.infer<
	typeof optimizationSplitChunksOptions
>;

const optimization = z.strictObject({
	moduleIds: z.enum(["named", "deterministic"]).optional(),
	chunkIds: z.enum(["named", "deterministic"]).optional(),
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
	nodeEnv: z.union([z.string(), z.literal(false)]).optional()
});
export type Optimization = z.infer<typeof optimization>;
//#endregion

//#region Experiments
const rspackFutureOptions = z.strictObject({
	newTreeshaking: z.boolean().optional(),
	disableApplyEntryLazily: z.boolean().optional(),
	bundlerInfo: z
		.strictObject({
			version: z.string().optional(),
			force: z
				.boolean()
				.or(z.array(z.enum(["version"])))
				.optional()
		})
		.optional()
});
export type RspackFutureOptions = z.infer<typeof rspackFutureOptions>;

const experiments = z.strictObject({
	lazyCompilation: z.boolean().optional(),
	asyncWebAssembly: z.boolean().optional(),
	outputModule: z.boolean().optional(),
	topLevelAwait: z.boolean().optional(),
	newSplitChunks: z
		.boolean()
		.optional()
		.refine(val => {
			if (val === false) {
				deprecatedWarn(
					`'experiments.newSplitChunks = ${JSON.stringify(
						val
					)}' has been deprecated, please switch to 'experiments.newSplitChunks = true' to use webpack's behavior.
 	See the discussion ${termlink(
		"here",
		"https://github.com/web-infra-dev/rspack/discussions/4168"
	)}`
				);
			}
			return true;
		}),
	css: z.boolean().optional(),
	futureDefaults: z.boolean().optional(),
	rspackFuture: rspackFutureOptions.optional()
});
export type Experiments = z.infer<typeof experiments>;
//#endregion

//#region Watch
const watch = z.boolean();
export type Watch = z.infer<typeof watch>;
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
});
export type WatchOptions = z.infer<typeof watchOptions>;
//#endregion

//#region DevServer
export interface DevServer extends webpackDevServer.Configuration {}
const devServer = z.custom<DevServer>();
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
	.array();
export type IgnoreWarnings = z.infer<typeof ignoreWarnings>;
//#endregion

//#region Profile
const profile = z.boolean();
export type Profile = z.infer<typeof profile>;
//#endregion

//#region Bail
const bail = z.boolean();
export type Bail = z.infer<typeof bail>;
//#endregion

//#region Builtins (deprecated)
const builtins = z.custom<oldBuiltins.Builtins>();
export type Builtins = z.infer<typeof builtins>;
//#endregion

export const rspackOptions = z.strictObject({
	name: name.optional(),
	dependencies: dependencies.optional(),
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
	ignoreWarnings: ignoreWarnings.optional(),
	watchOptions: watchOptions.optional(),
	watch: watch.optional(),
	stats: statsValue.optional(),
	snapshot: snapshotOptions.optional(),
	optimization: optimization.optional(),
	resolve: resolve.optional(),
	resolveLoader: resolve.optional(),
	plugins: plugins.optional(),
	devServer: devServer.optional(),
	builtins: builtins.optional(),
	module: moduleOptions.optional(),
	profile: profile.optional(),
	bail: bail.optional()
});
export type RspackOptions = z.infer<typeof rspackOptions>;
export type Configuration = RspackOptions;
