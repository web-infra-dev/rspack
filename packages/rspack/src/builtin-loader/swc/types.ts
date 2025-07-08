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
import { numberOrInfinity } from "../../config/utils";
import { z } from "../../config/zod";
import { memoize } from "../../util/memoize";
import type { CollectTypeScriptInfoOptions } from "./collectTypeScriptInfo";
import type { PluginImportOptions } from "./pluginImport";
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
		collectTypeScriptInfo?: CollectTypeScriptInfoOptions;
	};
};

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

export const getZodSwcLoaderOptionsSchema = memoize(() => {
	const ZodSwcEnvConfig = z
		.strictObject({
			mode: z.enum(["usage", "entry"]),
			debug: z.boolean(),
			dynamicImport: z.boolean(),
			loose: z.boolean(),
			bugfixes: z.boolean(),
			skip: z.string().array(),
			include: z.string().array(),
			exclude: z.string().array(),
			coreJs: z.string(),
			targets: z.any(),
			path: z.string(),
			shippedProposals: z.boolean(),
			forceAllTransforms: z.boolean()
		})
		.partial() satisfies z.ZodType<EnvConfig>;

	const ZodSwcAssumptions = z
		.strictObject({
			arrayLikeIsIterable: z.boolean(),
			constantReexports: z.boolean(),
			constantSuper: z.boolean(),
			enumerableModuleMeta: z.boolean(),
			ignoreFunctionLength: z.boolean(),
			ignoreFunctionName: z.boolean(),
			ignoreToPrimitiveHint: z.boolean(),
			iterableIsArray: z.boolean(),
			mutableTemplateObject: z.boolean(),
			noClassCalls: z.boolean(),
			noDocumentAll: z.boolean(),
			noIncompleteNsImportDetection: z.boolean(),
			noNewArrows: z.boolean(),
			objectRestNoSymbols: z.boolean(),
			privateFieldsAsProperties: z.boolean(),
			pureGetters: z.boolean(),
			setClassMethods: z.boolean(),
			setComputedProperties: z.boolean(),
			setPublicClassFields: z.boolean(),
			setSpreadProperties: z.boolean(),
			skipForOfIteratorClosing: z.boolean(),
			superIsCallableConstructor: z.boolean(),
			tsEnumIsReadonly: z.boolean()
		})
		.partial() satisfies z.ZodType<Assumptions>;

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
		z.int()
	]) satisfies z.ZodType<TerserEcmaVersion>;

	const ZodSwcJsFormatOptions = z
		.strictObject({
			asciiOnly: z.boolean(),
			beautify: z.boolean(),
			braces: z.boolean(),
			comments: z.literal("some").or(z.literal("all")).or(z.literal(false)),
			ecma: ZodSwcTerserEcmaVersion,
			indentLevel: z.int(),
			indentStart: z.int(),
			inlineScript: z.boolean(),
			keepNumbers: z.int(),
			keepQuotedProps: z.boolean(),
			maxLineLen: numberOrInfinity,
			preamble: z.string(),
			quoteKeys: z.boolean(),
			quoteStyle: z.boolean(),
			preserveAnnotations: z.boolean(),
			safari10: z.boolean(),
			semicolons: z.boolean(),
			shebang: z.boolean(),
			webkit: z.boolean(),
			wrapIife: z.boolean(),
			wrapFuncArgs: z.boolean()
		})
		.partial() satisfies z.ZodType<JsFormatOptions>;

	const ZodSwcTerserCompressOptions = z
		.strictObject({
			arguments: z.boolean(),
			arrows: z.boolean(),
			booleans: z.boolean(),
			booleans_as_integers: z.boolean(),
			collapse_vars: z.boolean(),
			comparisons: z.boolean(),
			computed_props: z.boolean(),
			conditionals: z.boolean(),
			dead_code: z.boolean(),
			defaults: z.boolean(),
			directives: z.boolean(),
			drop_console: z.boolean(),
			drop_debugger: z.boolean(),
			ecma: ZodSwcTerserEcmaVersion,
			evaluate: z.boolean(),
			expression: z.boolean(),
			global_defs: z.any(),
			hoist_funs: z.boolean(),
			hoist_props: z.boolean(),
			hoist_vars: z.boolean(),
			ie8: z.boolean(),
			if_return: z.boolean(),
			inline: z.literal(0).or(z.literal(1)).or(z.literal(2)).or(z.literal(3)),
			join_vars: z.boolean(),
			keep_classnames: z.boolean(),
			keep_fargs: z.boolean(),
			keep_fnames: z.boolean(),
			keep_infinity: z.boolean(),
			loops: z.boolean(),
			negate_iife: z.boolean(),
			passes: numberOrInfinity,
			properties: z.boolean(),
			pure_getters: z.any(),
			pure_funcs: z.string().array(),
			reduce_funcs: z.boolean(),
			reduce_vars: z.boolean(),
			sequences: z.any(),
			side_effects: z.boolean(),
			switches: z.boolean(),
			top_retain: z.any(),
			toplevel: z.any(),
			typeofs: z.boolean(),
			unsafe: z.boolean(),
			unsafe_passes: z.boolean(),
			unsafe_arrows: z.boolean(),
			unsafe_comps: z.boolean(),
			unsafe_function: z.boolean(),
			unsafe_math: z.boolean(),
			unsafe_symbols: z.boolean(),
			unsafe_methods: z.boolean(),
			unsafe_proto: z.boolean(),
			unsafe_regexp: z.boolean(),
			unsafe_undefined: z.boolean(),
			unused: z.boolean(),
			const_to_let: z.boolean(),
			module: z.boolean()
		})
		.partial() satisfies z.ZodType<TerserCompressOptions>;

	const ZodSwcTerserMangleOptions = z
		.strictObject({
			props: z.record(z.string(), z.any()),
			topLevel: z.boolean(),
			toplevel: z.boolean(),
			keepClassNames: z.boolean(),
			keep_classnames: z.boolean(),
			keepFnNames: z.boolean(),
			keep_fnames: z.boolean(),
			keepPrivateProps: z.boolean(),
			keep_private_props: z.boolean(),
			ie8: z.boolean(),
			safari10: z.boolean(),
			reserved: z.string().array()
		})
		.partial() satisfies z.ZodType<TerserMangleOptions>;

	const ZodSwcReactConfig = z
		.strictObject({
			pragma: z.string(),
			pragmaFrag: z.string(),
			throwIfNamespace: z.boolean(),
			development: z.boolean(),
			useBuiltins: z.boolean(),
			refresh: z.boolean().or(
				z
					.strictObject({
						refreshReg: z.string(),
						refreshSig: z.string(),
						emitFullSignatures: z.boolean()
					})
					.partial()
			),
			runtime: z.enum(["automatic", "classic"]),
			importSource: z.string()
		})
		.partial() satisfies z.ZodType<ReactConfig>;

	const ZodSwcConstModulesConfig = z.strictObject({
		globals: z.record(z.string(), z.record(z.string(), z.string())).optional()
	}) satisfies z.ZodType<ConstModulesConfig>;

	const ZodSwcGlobalPassOption = z
		.strictObject({
			vars: z.record(z.string(), z.string()),
			envs: z.union([z.string().array(), z.record(z.string(), z.string())]),
			typeofs: z.record(z.string(), z.string())
		})
		.partial() satisfies z.ZodType<GlobalPassOption>;

	const ZodSwcOptimizerConfig = z
		.strictObject({
			simplify: z.boolean(),
			globals: ZodSwcGlobalPassOption,
			jsonify: z.strictObject({
				minCost: numberOrInfinity
			})
		})
		.partial() satisfies z.ZodType<OptimizerConfig>;

	const ZodSwcTransformConfig = z
		.strictObject({
			react: ZodSwcReactConfig,
			constModules: ZodSwcConstModulesConfig,
			optimizer: ZodSwcOptimizerConfig,
			legacyDecorator: z.boolean(),
			decoratorMetadata: z.boolean(),
			decoratorVersion: z.enum(["2021-12", "2022-03"]),
			treatConstEnumAsEnum: z.boolean(),
			useDefineForClassFields: z.boolean(),
			verbatimModuleSyntax: z.boolean()
		})
		.partial() satisfies z.ZodType<TransformConfig>;

	const ZodSwcJsMinifyOptions = z
		.strictObject({
			compress: z.union([ZodSwcTerserCompressOptions, z.boolean()]),
			format: ZodSwcJsFormatOptions,
			mangle: z.union([ZodSwcTerserMangleOptions, z.boolean()]),
			ecma: ZodSwcTerserEcmaVersion,
			keep_classnames: z.boolean(),
			keep_fnames: z.boolean(),
			module: z.union([z.boolean(), z.literal("unknown")]),
			safari10: z.boolean(),
			toplevel: z.boolean(),
			sourceMap: z.boolean(),
			outputPath: z.string(),
			inlineSourcesContent: z.boolean()
		})
		.partial() satisfies z.ZodType<JsMinifyOptions>;

	const ZodSwcJscConfig = z
		.strictObject({
			assumptions: ZodSwcAssumptions,
			loose: z.boolean(),
			parser: ZodSwcParserConfig,
			transform: ZodSwcTransformConfig,
			externalHelpers: z.boolean(),
			target: ZodSwcJscTarget,
			keepClassNames: z.boolean(),
			experimental: z
				.strictObject({
					optimizeHygiene: z.boolean(),
					keepImportAttributes: z.boolean(),
					emitAssertForImportAttributes: z.boolean(),
					cacheRoot: z.string(),
					plugins: z.array(
						z.tuple([z.string(), z.record(z.string(), z.any())])
					),
					runPluginFirst: z.boolean(),
					disableBuiltinTransformsForInternalTesting: z.boolean(),
					emitIsolatedDts: z.boolean(),
					disableAllLints: z.boolean(),
					keepImportAssertions: z.boolean()
				})
				.partial(),
			baseUrl: z.string(),
			paths: z.record(z.string(), z.string().array()),
			minify: ZodSwcJsMinifyOptions,
			preserveAllComments: z.boolean(),
			output: z.strictObject({
				charset: z.enum(["utf8", "ascii"]).optional()
			})
		})
		.partial() satisfies z.ZodType<JscConfig>;

	const ZodSwcBaseModuleConfig = z
		.strictObject({
			strict: z.boolean(),
			strictMode: z.boolean(),
			lazy: z.union([z.boolean(), z.string().array()]),
			noInterop: z.boolean(),
			importInterop: z.enum(["swc", "babel", "node", "none"]),
			outFileExtension: z.enum(["js", "mjs", "cjs"]),
			exportInteropAnnotation: z.boolean(),
			ignoreDynamic: z.boolean(),
			allowTopLevelThis: z.boolean(),
			preserveImportMeta: z.boolean()
		})
		.partial() satisfies z.ZodType<BaseModuleConfig>;

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

	const ZodSwcConfig = z
		.strictObject({
			$schema: z.string(),
			test: z.string().or(z.string().array()),
			exclude: z.string().or(z.string().array()),
			env: ZodSwcEnvConfig,
			jsc: ZodSwcJscConfig,
			module: ZodSwcModuleConfig,
			minify: z.boolean(),
			sourceMaps: z.boolean().or(z.literal("inline")),
			inlineSourcesContent: z.boolean()
		})
		.partial() satisfies z.ZodType<Config>;

	const ZodSwcCollectTypeScriptInfo = z.strictObject({
		typeExports: z.boolean().optional(),
		exportedEnum: z.boolean().or(z.literal("const-only")).optional()
	}) satisfies z.ZodType<CollectTypeScriptInfoOptions>;

	const ZodSwcPluginImportConfig = z
		.strictObject({
			libraryName: z.string(),
			libraryDirectory: z.string().optional(),
			customName: z.string().optional(),
			customStyleName: z.string().optional(),
			style: z.string().or(z.boolean()).optional(),
			styleLibraryDirectory: z.string().optional(),
			camelToDashComponentName: z.boolean().optional(),
			transformToDefaultImport: z.boolean().optional(),
			ignoreEsComponent: z.string().array().optional(),
			ignoreStyleComponent: z.string().array().optional()
		})
		.array() satisfies z.ZodType<PluginImportOptions>;

	return ZodSwcConfig.extend({
		isModule: z.boolean().or(z.literal("unknown")),
		rspackExperiments: z
			.strictObject({
				import: ZodSwcPluginImportConfig,
				collectTypeScriptInfo: ZodSwcCollectTypeScriptInfo
			})
			.partial()
	}).partial() satisfies z.ZodType<SwcLoaderOptions>;
});
