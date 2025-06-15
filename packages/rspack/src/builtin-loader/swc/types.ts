import type {
	AmdConfig,
	BaseModuleConfig,
	CommonJsConfig,
	Config,
	ConstModulesConfig,
	EnvConfig,
	Es6Config,
	EsParserConfig,
	GlobalPassOption,
	JsFormatOptions,
	JsMinifyOptions,
	JscConfig,
	JscTarget,
	ModuleConfig,
	NodeNextConfig,
	OptimizerConfig,
	ParserConfig,
	ReactConfig,
	SystemjsConfig,
	TerserEcmaVersion,
	TerserMangleOptions,
	TransformConfig,
	TsParserConfig,
	UmdConfig
} from "@swc/types";
import type { Assumptions } from "@swc/types/assumptions";
import { z } from "zod";
import {
	type PluginImportOptions,
	ZodSwcPluginImportConfig
} from "./pluginImport";
export type SwcLoaderEnvConfig = EnvConfig;
export type SwcLoaderJscConfig = JscConfig;
export type SwcLoaderModuleConfig = ModuleConfig;
export type SwcLoaderParserConfig = ParserConfig;
export type SwcLoaderEsParserConfig = EsParserConfig;
export type SwcLoaderTsParserConfig = TsParserConfig;
export type SwcLoaderTransformConfig = TransformConfig;
export type SwcLoaderOptions = Config & {
	isModule?: boolean | "unknown";
	/**
	 * Experimental features provided by Rspack.
	 * @experimental
	 */
	rspackExperiments?: {
		import?: PluginImportOptions;
	};
};

const ZodSwcEnvConfig = z.strictObject({
	mode: z.enum(["usage", "entry"]).optional(),
	debug: z.boolean().optional(),
	dynamicImport: z.boolean().optional(),
	loose: z.boolean().optional(),
	bugfixes: z.boolean().optional(),
	skip: z.string().array().optional(),
	include: z.string().array().optional(),
	exclude: z.string().array().optional(),
	coreJs: z.string().optional(),
	targets: z.any().optional(),
	path: z.string().optional(),
	shippedProposals: z.boolean().optional(),
	forceAllTransforms: z.boolean().optional()
}) satisfies z.ZodType<EnvConfig>;

const ZodSwcAssumptions = z.strictObject({
	arrayLikeIsIterable: z.boolean().optional(),
	constantReexports: z.boolean().optional(),
	constantSuper: z.boolean().optional(),
	enumerableModuleMeta: z.boolean().optional(),
	ignoreFunctionLength: z.boolean().optional(),
	ignoreFunctionName: z.boolean().optional(),
	ignoreToPrimitiveHint: z.boolean().optional(),
	iterableIsArray: z.boolean().optional(),
	mutableTemplateObject: z.boolean().optional(),
	noClassCalls: z.boolean().optional(),
	noDocumentAll: z.boolean().optional(),
	noIncompleteNsImportDetection: z.boolean().optional(),
	noNewArrows: z.boolean().optional(),
	objectRestNoSymbols: z.boolean().optional(),
	privateFieldsAsProperties: z.boolean().optional(),
	pureGetters: z.boolean().optional(),
	setClassMethods: z.boolean().optional(),
	setComputedProperties: z.boolean().optional(),
	setPublicClassFields: z.boolean().optional(),
	setSpreadProperties: z.boolean().optional(),
	skipForOfIteratorClosing: z.boolean().optional(),
	superIsCallableConstructor: z.boolean().optional(),
	tsEnumIsReadonly: z.boolean().optional()
}) satisfies z.ZodType<Assumptions>;

const ZodSwcParserConfig = z.strictObject({
	syntax: z.enum(["typescript", "ecmascript"]),
	// typescript only
	tsx: z.boolean().optional(),
	decorators: z.boolean().optional(),
	dynamicImport: z.boolean().optional(),
	// ecmascript only
	jsx: z.boolean().optional(),
	numericSeparator: z.boolean().optional(),
	classPrivateProperty: z.boolean().optional(),
	privateMethod: z.boolean().optional(),
	classProperty: z.boolean().optional(),
	functionBind: z.boolean().optional(),
	// decorators: z.boolean().optional(),
	decoratorsBeforeExport: z.boolean().optional(),
	exportDefaultFrom: z.boolean().optional(),
	exportNamespaceFrom: z.boolean().optional(),
	// dynamicImport: z.boolean().optional(),
	nullishCoalescing: z.boolean().optional(),
	optionalChaining: z.boolean().optional(),
	importMeta: z.boolean().optional(),
	topLevelAwait: z.boolean().optional(),
	importAssertions: z.boolean().optional(),
	importAttributes: z.boolean().optional(),
	allowSuperOutsideMethod: z.boolean().optional(),
	allowReturnOutsideFunction: z.boolean().optional(),
	autoAccessors: z.boolean().optional(),
	explicitResourceManagement: z.boolean().optional()
});

const ZodSwcJscTarget = z.enum([
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
	"esnext"
]) satisfies z.ZodType<JscTarget>;

const ZodSwcTerserEcmaVersion = z.union([
	z.literal(5),
	z.literal(2015),
	z.literal(2016),
	z.string(),
	z.number()
]) satisfies z.ZodType<TerserEcmaVersion>;

const ZodSwcJsFormatOptions = z.strictObject({
	asciiOnly: z.boolean().optional(),
	beautify: z.boolean().optional(),
	braces: z.boolean().optional(),
	comments: z
		.literal("some")
		.or(z.literal("all"))
		.or(z.literal(false))
		.optional(),
	ecma: ZodSwcTerserEcmaVersion.optional(),
	indentLevel: z.number().optional(),
	indentStart: z.number().optional(),
	inlineScript: z.boolean().optional(),
	keepNumbers: z.number().optional(),
	keepQuotedProps: z.boolean().optional(),
	maxLineLen: z.number().optional(),
	preamble: z.string().optional(),
	quoteKeys: z.boolean().optional(),
	quoteStyle: z.boolean().optional(),
	preserveAnnotations: z.boolean().optional(),
	safari10: z.boolean().optional(),
	semicolons: z.boolean().optional(),
	shebang: z.boolean().optional(),
	webkit: z.boolean().optional(),
	wrapIife: z.boolean().optional(),
	wrapFuncArgs: z.boolean().optional()
}) satisfies z.ZodType<JsFormatOptions>;
export interface TerserCompressOptions {
	arguments?: boolean;
	arrows?: boolean;
	booleans?: boolean;
	booleans_as_integers?: boolean;
	collapse_vars?: boolean;
	comparisons?: boolean;
	computed_props?: boolean;
	conditionals?: boolean;
	dead_code?: boolean;
	defaults?: boolean;
	directives?: boolean;
	drop_console?: boolean;
	drop_debugger?: boolean;
	ecma?: TerserEcmaVersion;
	evaluate?: boolean;
	expression?: boolean;
	global_defs?: any;
	hoist_funs?: boolean;
	hoist_props?: boolean;
	hoist_vars?: boolean;
	ie8?: boolean;
	if_return?: boolean;
	inline?: 0 | 1 | 2 | 3;
	join_vars?: boolean;
	keep_classnames?: boolean;
	keep_fargs?: boolean;
	keep_fnames?: boolean;
	keep_infinity?: boolean;
	loops?: boolean;
	negate_iife?: boolean;
	passes?: number;
	properties?: boolean;
	pure_getters?: any;
	pure_funcs?: string[];
	reduce_funcs?: boolean;
	reduce_vars?: boolean;
	sequences?: any;
	side_effects?: boolean;
	switches?: boolean;
	top_retain?: any;
	toplevel?: any;
	typeofs?: boolean;
	unsafe?: boolean;
	unsafe_passes?: boolean;
	unsafe_arrows?: boolean;
	unsafe_comps?: boolean;
	unsafe_function?: boolean;
	unsafe_math?: boolean;
	unsafe_symbols?: boolean;
	unsafe_methods?: boolean;
	unsafe_proto?: boolean;
	unsafe_regexp?: boolean;
	unsafe_undefined?: boolean;
	unused?: boolean;
	const_to_let?: boolean;
	module?: boolean;
}

const ZodSwcTerserCompressOptions = z.strictObject({
	arguments: z.boolean().optional(),
	arrows: z.boolean().optional(),
	booleans: z.boolean().optional(),
	booleans_as_integers: z.boolean().optional(),
	collapse_vars: z.boolean().optional(),
	comparisons: z.boolean().optional(),
	computed_props: z.boolean().optional(),
	conditionals: z.boolean().optional(),
	dead_code: z.boolean().optional(),
	defaults: z.boolean().optional(),
	directives: z.boolean().optional(),
	drop_console: z.boolean().optional(),
	drop_debugger: z.boolean().optional(),
	ecma: ZodSwcTerserEcmaVersion.optional(),
	evaluate: z.boolean().optional(),
	expression: z.boolean().optional(),
	global_defs: z.any().optional(),
	hoist_funs: z.boolean().optional(),
	hoist_props: z.boolean().optional(),
	hoist_vars: z.boolean().optional(),
	ie8: z.boolean().optional(),
	if_return: z.boolean().optional(),
	inline: z
		.literal(0)
		.or(z.literal(1))
		.or(z.literal(2))
		.or(z.literal(3))
		.optional(),
	join_vars: z.boolean().optional(),
	keep_classnames: z.boolean().optional(),
	keep_fargs: z.boolean().optional(),
	keep_fnames: z.boolean().optional(),
	keep_infinity: z.boolean().optional(),
	loops: z.boolean().optional(),
	negate_iife: z.boolean().optional(),
	passes: z.number().optional(),
	properties: z.boolean().optional(),
	pure_getters: z.any().optional(),
	pure_funcs: z.string().array().optional(),
	reduce_funcs: z.boolean().optional(),
	reduce_vars: z.boolean().optional(),
	sequences: z.any().optional(),
	side_effects: z.boolean().optional(),
	switches: z.boolean().optional(),
	top_retain: z.any().optional(),
	toplevel: z.any().optional(),
	typeofs: z.boolean().optional(),
	unsafe: z.boolean().optional(),
	unsafe_passes: z.boolean().optional(),
	unsafe_arrows: z.boolean().optional(),
	unsafe_comps: z.boolean().optional(),
	unsafe_function: z.boolean().optional(),
	unsafe_math: z.boolean().optional(),
	unsafe_symbols: z.boolean().optional(),
	unsafe_methods: z.boolean().optional(),
	unsafe_proto: z.boolean().optional(),
	unsafe_regexp: z.boolean().optional(),
	unsafe_undefined: z.boolean().optional(),
	unused: z.boolean().optional(),
	const_to_let: z.boolean().optional(),
	module: z.boolean().optional()
}) satisfies z.ZodType<TerserCompressOptions>;

const ZodSwcTerserMangleOptions = z.strictObject({
	props: z.record(z.string(), z.any()).optional(),
	topLevel: z.boolean().optional(),
	toplevel: z.boolean().optional(),
	keepClassNames: z.boolean().optional(),
	keep_classnames: z.boolean().optional(),
	keepFnNames: z.boolean().optional(),
	keep_fnames: z.boolean().optional(),
	keepPrivateProps: z.boolean().optional(),
	keep_private_props: z.boolean().optional(),
	ie8: z.boolean().optional(),
	safari10: z.boolean().optional(),
	reserved: z.string().array().optional()
}) satisfies z.ZodType<TerserMangleOptions>;

const ZodSwcReactConfig = z.strictObject({
	pragma: z.string().optional(),
	pragmaFrag: z.string().optional(),
	throwIfNamespace: z.boolean().optional(),
	development: z.boolean().optional(),
	useBuiltins: z.boolean().optional(),
	refresh: z
		.boolean()
		.or(
			z.strictObject({
				refreshReg: z.string().optional(),
				refreshSig: z.string().optional(),
				emitFullSignatures: z.boolean().optional()
			})
		)
		.optional(),
	runtime: z.enum(["automatic", "classic"]).optional(),
	importSource: z.string().optional()
}) satisfies z.ZodType<ReactConfig>;

const ZodSwcConstModulesConfig = z.strictObject({
	globals: z.record(z.string(), z.record(z.string(), z.string())).optional()
}) satisfies z.ZodType<ConstModulesConfig>;

const ZodSwcGlobalPassOption = z.strictObject({
	vars: z.record(z.string(), z.string()).optional(),
	envs: z
		.union([z.string().array(), z.record(z.string(), z.string())])
		.optional(),
	typeofs: z.record(z.string(), z.string()).optional()
}) satisfies z.ZodType<GlobalPassOption>;

const ZodSwcOptimizerConfig = z.strictObject({
	simplify: z.boolean().optional(),
	globals: ZodSwcGlobalPassOption.optional(),
	jsonify: z
		.strictObject({
			minCost: z.number()
		})
		.optional()
}) satisfies z.ZodType<OptimizerConfig>;

const ZodSwcTransformConfig = z.strictObject({
	react: ZodSwcReactConfig.optional(),
	constModules: ZodSwcConstModulesConfig.optional(),
	optimizer: ZodSwcOptimizerConfig.optional(),
	legacyDecorator: z.boolean().optional(),
	decoratorMetadata: z.boolean().optional(),
	decoratorVersion: z.enum(["2021-12", "2022-03"]).optional(),
	treatConstEnumAsEnum: z.boolean().optional(),
	useDefineForClassFields: z.boolean().optional(),
	verbatimModuleSyntax: z.boolean().optional()
}) satisfies z.ZodType<TransformConfig>;

const ZodSwcJsMinifyOptions = z.strictObject({
	compress: z.union([ZodSwcTerserCompressOptions, z.boolean()]).optional(),
	format: ZodSwcJsFormatOptions.optional(),
	mangle: z.union([ZodSwcTerserMangleOptions, z.boolean()]).optional(),
	ecma: ZodSwcTerserEcmaVersion.optional(),
	keep_classnames: z.boolean().optional(),
	keep_fnames: z.boolean().optional(),
	module: z.union([z.boolean(), z.literal("unknown")]).optional(),
	safari10: z.boolean().optional(),
	toplevel: z.boolean().optional(),
	sourceMap: z.boolean().optional(),
	outputPath: z.string().optional(),
	inlineSourcesContent: z.boolean().optional()
}) satisfies z.ZodType<JsMinifyOptions>;

const ZodSwcJscConfig = z.strictObject({
	assumptions: ZodSwcAssumptions.optional(),
	loose: z.boolean().optional(),
	parser: ZodSwcParserConfig.optional(),
	transform: ZodSwcTransformConfig.optional(),
	externalHelpers: z.boolean().optional(),
	target: ZodSwcJscTarget.optional(),
	keepClassNames: z.boolean().optional(),
	experimental: z
		.strictObject({
			optimizeHygiene: z.boolean().optional(),
			keepImportAttributes: z.boolean().optional(),
			emitAssertForImportAttributes: z.boolean().optional(),
			cacheRoot: z.string().optional(),
			plugins: z
				.array(z.tuple([z.string(), z.record(z.string(), z.any())]))
				.optional(),
			runPluginFirst: z.boolean().optional(),
			disableBuiltinTransformsForInternalTesting: z.boolean().optional(),
			emitIsolatedDts: z.boolean().optional(),
			disableAllLints: z.boolean().optional(),
			keepImportAssertions: z.boolean().optional()
		})
		.optional(),
	baseUrl: z.string().optional(),
	paths: z.record(z.string(), z.string().array()).optional(),
	minify: ZodSwcJsMinifyOptions.optional(),
	preserveAllComments: z.boolean().optional(),
	output: z
		.strictObject({
			charset: z.enum(["utf8", "ascii"]).optional()
		})
		.optional()
}) satisfies z.ZodType<JscConfig>;

const ZodSwcBaseModuleConfig = z.strictObject({
	strict: z.boolean().optional(),
	strictMode: z.boolean().optional(),
	lazy: z.union([z.boolean(), z.string().array()]).optional(),
	noInterop: z.boolean().optional(),
	importInterop: z.enum(["swc", "babel", "node", "none"]).optional(),
	outFileExtension: z.enum(["js", "mjs", "cjs"]).optional(),
	exportInteropAnnotation: z.boolean().optional(),
	ignoreDynamic: z.boolean().optional(),
	allowTopLevelThis: z.boolean().optional(),
	preserveImportMeta: z.boolean().optional()
}) satisfies z.ZodType<BaseModuleConfig>;

const ZodSwcEs6Config = ZodSwcBaseModuleConfig.extend({
	type: z.literal("es6")
}) satisfies z.ZodType<Es6Config>;

const ZodSwcNodeNextConfig = ZodSwcBaseModuleConfig.extend({
	type: z.literal("nodenext")
}) satisfies z.ZodType<NodeNextConfig>;

const ZodSwcCommonJsConfig = ZodSwcBaseModuleConfig.extend({
	type: z.literal("commonjs")
}) satisfies z.ZodType<CommonJsConfig>;

const ZodSwcUmdConfig = ZodSwcBaseModuleConfig.extend({
	type: z.literal("umd"),
	globals: z.record(z.string(), z.string()).optional()
}) satisfies z.ZodType<UmdConfig>;

const ZodSwcAmdConfig = ZodSwcBaseModuleConfig.extend({
	type: z.literal("amd"),
	moduleId: z.string().optional()
}) satisfies z.ZodType<AmdConfig>;

const ZodSwcSystemjsConfig = z.strictObject({
	type: z.literal("systemjs"),
	allowTopLevelThis: z.boolean().optional()
}) satisfies z.ZodType<SystemjsConfig>;

const ZodSwcModuleConfig = z.union([
	ZodSwcEs6Config,
	ZodSwcCommonJsConfig,
	ZodSwcUmdConfig,
	ZodSwcAmdConfig,
	ZodSwcNodeNextConfig,
	ZodSwcSystemjsConfig
]) satisfies z.ZodType<ModuleConfig>;

const ZodSwcConfig = z.strictObject({
	$schema: z.string().optional(),
	test: z.string().or(z.string().array()).optional(),
	exclude: z.string().or(z.string().array()).optional(),
	env: ZodSwcEnvConfig.optional(),
	jsc: ZodSwcJscConfig.optional(),
	module: ZodSwcModuleConfig.optional(),
	minify: z.boolean().optional(),
	sourceMaps: z.boolean().or(z.literal("inline")).optional(),
	inlineSourcesContent: z.boolean().optional()
}) satisfies z.ZodType<Config>;

export const ZodSwcLoaderOptions = ZodSwcConfig.extend({
	isModule: z.boolean().or(z.literal("unknown")).optional(),
	rspackExperiments: z
		.strictObject({
			import: ZodSwcPluginImportConfig.optional()
		})
		.optional()
}) satisfies z.ZodType<SwcLoaderOptions>;
