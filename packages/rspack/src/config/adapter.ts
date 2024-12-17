import assert from "node:assert";
import {
	type JsLibraryName,
	type JsLibraryOptions,
	type RawAssetGeneratorDataUrlFnCtx,
	type RawAssetGeneratorOptions,
	type RawAssetInlineGeneratorOptions,
	type RawAssetParserDataUrl,
	type RawAssetParserOptions,
	type RawAssetResourceGeneratorOptions,
	type RawCssAutoGeneratorOptions,
	type RawCssAutoParserOptions,
	type RawCssGeneratorOptions,
	type RawCssModuleGeneratorOptions,
	type RawCssModuleParserOptions,
	type RawCssParserOptions,
	type RawExperiments,
	type RawFuncUseCtx,
	type RawGeneratorOptions,
	type RawIncremental,
	type RawJavascriptParserOptions,
	type RawModuleRule,
	type RawModuleRuleUse,
	type RawOptions,
	type RawParserOptions,
	type RawRspackFuture,
	type RawRuleSetCondition,
	RawRuleSetConditionType,
	type RawRuleSetLogicalConditions
} from "@rspack/binding";

import type { Compiler } from "../Compiler";
import { Module } from "../Module";
import { normalizeStatsPreset } from "../Stats";
import { isNil } from "../util";
import { parseResource } from "../util/identifier";
import {
	type ComposeJsUseOptions,
	type LoaderContext,
	type LoaderDefinition,
	type LoaderDefinitionFunction,
	createRawModuleRuleUses
} from "./adapterRuleUse";
import type {
	ExperimentCacheNormalized,
	ExperimentsNormalized,
	ModuleOptionsNormalized,
	OutputNormalized,
	RspackOptionsNormalized
} from "./normalization";
import type {
	AssetGeneratorDataUrl,
	AssetGeneratorOptions,
	AssetInlineGeneratorOptions,
	AssetParserDataUrl,
	AssetParserOptions,
	AssetResourceGeneratorOptions,
	ChunkLoading,
	CrossOriginLoading,
	CssAutoGeneratorOptions,
	CssGeneratorOptions,
	CssParserOptions,
	GeneratorOptionsByModuleType,
	Incremental,
	JavascriptParserOptions,
	LibraryName,
	LibraryOptions,
	Node,
	Optimization,
	ParserOptionsByModuleType,
	Resolve,
	RspackFutureOptions,
	RuleSetCondition,
	RuleSetLogicalConditions,
	RuleSetRule,
	SnapshotOptions,
	StatsValue,
	Target
} from "./types";

export type { LoaderContext, LoaderDefinition, LoaderDefinitionFunction };

export const getRawOptions = (
	options: RspackOptionsNormalized,
	compiler: Compiler
): RawOptions => {
	assert(
		!isNil(options.context) && !isNil(options.devtool) && !isNil(options.cache),
		"context, devtool, cache should not be nil after defaults"
	);
	const devtool = options.devtool === false ? "" : options.devtool;
	const mode = options.mode;
	const experiments = getRawExperiments(options.experiments);
	return {
		mode,
		target: getRawTarget(options.target),
		context: options.context,
		output: getRawOutput(options.output),
		resolve: getRawResolve(options.resolve),
		resolveLoader: getRawResolve(options.resolveLoader),
		module: getRawModule(options.module, {
			compiler,
			devtool,
			mode,
			context: options.context,
			experiments
		}),
		devtool,
		optimization: getRawOptimization(options.optimization),
		stats: getRawStats(options.stats),
		snapshot: getRawSnapshotOptions(options.snapshot),
		cache: {
			type: options.cache ? "memory" : "disable",
			// TODO: implement below cache options
			maxGenerations: 0,
			maxAge: 0,
			profile: false,
			buildDependencies: [],
			cacheDirectory: "",
			cacheLocation: "",
			name: "",
			version: ""
		},
		experiments,
		node: getRawNode(options.node),
		// SAFETY: applied default value in `applyRspackOptionsDefaults`.
		profile: options.profile!,
		amd: options.amd,
		// SAFETY: applied default value in `applyRspackOptionsDefaults`.
		bail: options.bail!,
		__references: {}
	};
};

function getRawTarget(target: Target | undefined): RawOptions["target"] {
	if (!target) {
		return [];
	}
	if (typeof target === "string") {
		return [target];
	}
	return target;
}

function getRawExtensionAlias(
	alias: Resolve["extensionAlias"] = {}
): RawOptions["resolve"]["extensionAlias"] {
	const entries = Object.entries(alias).map(([key, value]) => {
		if (Array.isArray(value)) {
			return [key, value];
		}
		return [key, [value]];
	});
	return Object.fromEntries(entries);
}

function getRawAlias(
	alias: Resolve["alias"] = {}
): RawOptions["resolve"]["alias"] {
	return Object.entries(alias).map(([key, value]) => ({
		path: key,
		redirect: Array.isArray(value) ? value : [value]
	}));
}

function getRawResolveByDependency(
	byDependency: Resolve["byDependency"]
): RawOptions["resolve"]["byDependency"] {
	if (byDependency === undefined) return byDependency;
	return Object.fromEntries(
		Object.entries(byDependency).map(([k, v]) => [k, getRawResolve(v)])
	);
}

function getRawTsConfig(
	tsConfig: Resolve["tsConfig"]
): RawOptions["resolve"]["tsconfig"] {
	assert(
		typeof tsConfig !== "string",
		"should resolve string tsConfig in normalization"
	);
	if (tsConfig === undefined) return tsConfig;
	const { configFile, references } = tsConfig;
	return {
		configFile,
		referencesType:
			references === "auto" ? "auto" : references ? "manual" : "disabled",
		references: references === "auto" ? undefined : references
	};
}

export function getRawResolve(resolve: Resolve): RawOptions["resolve"] {
	return {
		...resolve,
		alias: getRawAlias(resolve.alias),
		fallback: getRawAlias(resolve.fallback),
		extensionAlias: getRawExtensionAlias(resolve.extensionAlias) as Record<
			string,
			Array<string>
		>,
		tsconfig: getRawTsConfig(resolve.tsConfig),
		byDependency: getRawResolveByDependency(resolve.byDependency)
	};
}

function getRawCrossOriginLoading(
	crossOriginLoading: CrossOriginLoading
): RawOptions["output"]["crossOriginLoading"] {
	if (typeof crossOriginLoading === "boolean") {
		return { type: "bool", boolPayload: crossOriginLoading };
	}
	return { type: "string", stringPayload: crossOriginLoading };
}

function getRawOutput(output: OutputNormalized): RawOptions["output"] {
	const chunkLoading = output.chunkLoading!;
	const wasmLoading = output.wasmLoading!;
	const workerChunkLoading = output.workerChunkLoading!;
	const workerWasmLoading = output.workerWasmLoading!;
	return {
		path: output.path!,
		pathinfo: output.pathinfo!,
		publicPath: output.publicPath!,
		clean: output.clean!,
		assetModuleFilename: output.assetModuleFilename!,
		filename: output.filename!,
		chunkFilename: output.chunkFilename!,
		chunkLoading: getRawChunkLoading(chunkLoading),
		crossOriginLoading: getRawCrossOriginLoading(output.crossOriginLoading!),
		cssFilename: output.cssFilename!,
		cssChunkFilename: output.cssChunkFilename!,
		hotUpdateChunkFilename: output.hotUpdateChunkFilename!,
		hotUpdateMainFilename: output.hotUpdateMainFilename!,
		hotUpdateGlobal: output.hotUpdateGlobal!,
		uniqueName: output.uniqueName!,
		chunkLoadingGlobal: output.chunkLoadingGlobal!,
		enabledLibraryTypes: output.enabledLibraryTypes,
		library: output.library && getRawLibrary(output.library),
		strictModuleErrorHandling: output.strictModuleErrorHandling!,
		globalObject: output.globalObject!,
		importFunctionName: output.importFunctionName!,
		importMetaName: output.importMetaName!,
		iife: output.iife!,
		module: output.module!,
		wasmLoading: wasmLoading === false ? "false" : wasmLoading,
		enabledWasmLoadingTypes: output.enabledWasmLoadingTypes!,
		enabledChunkLoadingTypes: output.enabledChunkLoadingTypes!,
		webassemblyModuleFilename: output.webassemblyModuleFilename!,
		trustedTypes: output.trustedTypes!,
		sourceMapFilename: output.sourceMapFilename!,
		hashFunction: output.hashFunction!,
		hashDigest: output.hashDigest!,
		hashDigestLength: output.hashDigestLength!,
		hashSalt: output.hashSalt!,
		asyncChunks: output.asyncChunks!,
		workerChunkLoading:
			workerChunkLoading === false ? "false" : workerChunkLoading,
		workerWasmLoading:
			workerWasmLoading === false ? "false" : workerWasmLoading,
		workerPublicPath: output.workerPublicPath!,
		scriptType: output.scriptType === false ? "false" : output.scriptType!,
		charset: output.charset!,
		chunkLoadTimeout: output.chunkLoadTimeout!,
		environment: output.environment!,
		compareBeforeEmit: output.compareBeforeEmit!
	};
}

export function getRawLibrary(library: LibraryOptions): JsLibraryOptions {
	const {
		type,
		name,
		export: libraryExport,
		umdNamedDefine,
		auxiliaryComment,
		amdContainer
	} = library;
	return {
		amdContainer,
		auxiliaryComment:
			typeof auxiliaryComment === "string"
				? {
						commonjs: auxiliaryComment,
						commonjs2: auxiliaryComment,
						amd: auxiliaryComment,
						root: auxiliaryComment
					}
				: auxiliaryComment,
		libraryType: type,
		name: isNil(name) ? name : getRawLibraryName(name),
		export:
			Array.isArray(libraryExport) || libraryExport == null
				? libraryExport
				: [libraryExport],
		umdNamedDefine
	};
}

function getRawLibraryName(name: LibraryName): JsLibraryName {
	if (typeof name === "string") {
		return {
			type: "string",
			stringPayload: name
		};
	}
	if (Array.isArray(name)) {
		return {
			type: "array",
			arrayPayload: name
		};
	}
	if (typeof name === "object" && !Array.isArray(name)) {
		return {
			type: "umdObject",
			umdObjectPayload: {
				commonjs: name.commonjs,
				root:
					Array.isArray(name.root) || isNil(name.root)
						? name.root
						: [name.root],
				amd: name.amd
			}
		};
	}
	throw new Error("unreachable");
}

function getRawModule(
	module: ModuleOptionsNormalized,
	options: ComposeJsUseOptions
): RawOptions["module"] {
	assert(
		!isNil(module.defaultRules),
		"module.defaultRules should not be nil after defaults"
	);
	// "..." in defaultRules will be flatten in `applyModuleDefaults`, and "..." in rules is empty, so it's safe to use `as RuleSetRule[]` at here
	const ruleSet = [
		{ rules: module.defaultRules as RuleSetRule[] },
		{ rules: module.rules as RuleSetRule[] }
	];
	const rules = ruleSet.map((rule, index) =>
		getRawModuleRule(rule, `ruleSet[${index}]`, options, "javascript/auto")
	);
	return {
		rules,
		parser: getRawParserOptionsMap(module.parser),
		generator: getRawGeneratorOptionsMap(module.generator),
		noParse: module.noParse
	};
}

function tryMatch(payload: string, condition: RuleSetCondition): boolean {
	if (typeof condition === "string") {
		return payload.startsWith(condition);
	}

	if (condition instanceof RegExp) {
		return condition.test(payload);
	}

	if (typeof condition === "function") {
		return condition(payload);
	}

	if (Array.isArray(condition)) {
		return condition.some(c => tryMatch(payload, c));
	}

	if (condition && typeof condition === "object") {
		if (condition.and) {
			return condition.and.every(c => tryMatch(payload, c));
		}

		if (condition.or) {
			return condition.or.some(c => tryMatch(payload, c));
		}

		if (condition.not) {
			return !tryMatch(payload, condition.not);
		}
	}

	return false;
}

const getRawModuleRule = (
	rule: RuleSetRule,
	path: string,
	options: ComposeJsUseOptions,
	upperType: string
): RawModuleRule => {
	// Rule.loader is a shortcut to Rule.use: [ { loader } ].
	// See: https://webpack.js.org/configuration/module/#ruleloader
	if (rule.loader) {
		rule.use = [
			{
				loader: rule.loader,
				options: rule.options
			}
		];
	}
	let funcUse: undefined | ((rawContext: RawFuncUseCtx) => RawModuleRuleUse[]);
	if (typeof rule.use === "function") {
		const use = rule.use;
		funcUse = (rawContext: RawFuncUseCtx) => {
			const context = {
				...rawContext,
				compiler: options.compiler
			};
			const uses = use(context);

			return createRawModuleRuleUses(uses ?? [], `${path}.use`, options);
		};
	}

	const rawModuleRule: RawModuleRule = {
		test: rule.test ? getRawRuleSetCondition(rule.test) : undefined,
		include: rule.include ? getRawRuleSetCondition(rule.include) : undefined,
		exclude: rule.exclude ? getRawRuleSetCondition(rule.exclude) : undefined,
		issuer: rule.issuer ? getRawRuleSetCondition(rule.issuer) : undefined,
		issuerLayer: rule.issuerLayer
			? getRawRuleSetCondition(rule.issuerLayer)
			: undefined,
		dependency: rule.dependency
			? getRawRuleSetCondition(rule.dependency)
			: undefined,
		descriptionData: rule.descriptionData
			? Object.fromEntries(
					Object.entries(rule.descriptionData).map(([k, v]) => [
						k,
						getRawRuleSetCondition(v)
					])
				)
			: undefined,
		with: rule.with
			? Object.fromEntries(
					Object.entries(rule.with).map(([k, v]) => [
						k,
						getRawRuleSetCondition(v)
					])
				)
			: undefined,
		resource: rule.resource ? getRawRuleSetCondition(rule.resource) : undefined,
		resourceQuery: rule.resourceQuery
			? getRawRuleSetCondition(rule.resourceQuery)
			: undefined,
		resourceFragment: rule.resourceFragment
			? getRawRuleSetCondition(rule.resourceFragment)
			: undefined,
		scheme: rule.scheme ? getRawRuleSetCondition(rule.scheme) : undefined,
		mimetype: rule.mimetype ? getRawRuleSetCondition(rule.mimetype) : undefined,
		sideEffects: rule.sideEffects,
		use:
			typeof rule.use === "function"
				? funcUse
				: createRawModuleRuleUses(rule.use ?? [], `${path}.use`, options),
		type: rule.type,
		layer: rule.layer,
		parser: rule.parser
			? getRawParserOptions(rule.parser, rule.type ?? upperType)
			: undefined,
		generator: rule.generator
			? getRawGeneratorOptions(rule.generator, rule.type ?? upperType)
			: undefined,
		resolve: rule.resolve ? getRawResolve(rule.resolve) : undefined,
		oneOf: rule.oneOf
			? rule.oneOf
					.filter(Boolean)
					.map((rule, index) =>
						getRawModuleRule(
							rule as RuleSetRule,
							`${path}.oneOf[${index}]`,
							options,
							(rule as RuleSetRule).type ?? upperType
						)
					)
			: undefined,
		rules: rule.rules
			? rule.rules
					.filter(Boolean)
					.map((rule, index) =>
						getRawModuleRule(
							rule as RuleSetRule,
							`${path}.rules[${index}]`,
							options,
							(rule as RuleSetRule).type ?? upperType
						)
					)
			: undefined,
		enforce: rule.enforce
	};

	// Function calls may contain side-effects when interoperating with single-threaded environment.
	// In order to mitigate the issue, Rspack tries to merge these calls together.
	// See: https://github.com/web-infra-dev/rspack/issues/4003#issuecomment-1689662380
	if (
		typeof rule.test === "function" ||
		typeof rule.resource === "function" ||
		typeof rule.resourceQuery === "function" ||
		typeof rule.resourceFragment === "function"
	) {
		delete rawModuleRule.test;
		delete rawModuleRule.resource;
		delete rawModuleRule.resourceQuery;
		delete rawModuleRule.resourceFragment;

		rawModuleRule.rspackResource = getRawRuleSetCondition(
			resourceQueryFragment => {
				const { path, query, fragment } = parseResource(resourceQueryFragment);

				if (rule.test && !tryMatch(path, rule.test)) {
					return false;
				}
				if (rule.resource && !tryMatch(path, rule.resource)) {
					return false;
				}

				if (rule.resourceQuery && !tryMatch(query, rule.resourceQuery)) {
					return false;
				}

				if (
					rule.resourceFragment &&
					!tryMatch(fragment, rule.resourceFragment)
				) {
					return false;
				}

				return true;
			}
		);
	}
	return rawModuleRule;
};

function getRawRuleSetCondition(
	condition: RuleSetCondition
): RawRuleSetCondition {
	if (typeof condition === "string") {
		return {
			type: RawRuleSetConditionType.string,
			string: condition
		};
	}
	if (condition instanceof RegExp) {
		return {
			type: RawRuleSetConditionType.regexp,
			regexp: condition
		};
	}
	if (typeof condition === "function") {
		return {
			type: RawRuleSetConditionType.func,
			func: condition
		};
	}
	if (Array.isArray(condition)) {
		return {
			type: RawRuleSetConditionType.array,
			array: condition.map(i => getRawRuleSetCondition(i))
		};
	}
	if (typeof condition === "object" && condition !== null) {
		return {
			type: RawRuleSetConditionType.logical,
			logical: [getRawRuleSetLogicalConditions(condition)]
		};
	}
	throw new Error(
		"unreachable: condition should be one of string, RegExp, Array, Object"
	);
}

function getRawRuleSetLogicalConditions(
	logical: RuleSetLogicalConditions
): RawRuleSetLogicalConditions {
	return {
		and: logical.and
			? logical.and.map(i => getRawRuleSetCondition(i))
			: undefined,
		or: logical.or ? logical.or.map(i => getRawRuleSetCondition(i)) : undefined,
		not: logical.not ? getRawRuleSetCondition(logical.not) : undefined
	};
}

function getRawParserOptionsMap(
	parser: ParserOptionsByModuleType
): Record<string, RawParserOptions> {
	return Object.fromEntries(
		Object.entries(parser)
			.map(([k, v]) => [k, getRawParserOptions(v, k)])
			.filter(([k, v]) => v !== undefined)
	);
}

function getRawGeneratorOptionsMap(
	generator: GeneratorOptionsByModuleType
): Record<string, RawGeneratorOptions> {
	return Object.fromEntries(
		Object.entries(generator)
			.map(([k, v]) => [k, getRawGeneratorOptions(v, k)])
			.filter(([k, v]) => v !== undefined)
	);
}

function getRawParserOptions(
	parser: { [k: string]: any },
	type: string
): RawParserOptions | undefined {
	if (type === "asset") {
		return {
			type: "asset",
			asset: getRawAssetParserOptions(parser)
		};
	}
	if (type === "javascript") {
		return {
			type: "javascript",
			javascript: getRawJavascriptParserOptions(parser)
		};
	}
	if (type === "javascript/auto") {
		return {
			type: "javascript/auto",
			javascript: getRawJavascriptParserOptions(parser)
		};
	}
	if (type === "javascript/dynamic") {
		return {
			type: "javascript/dynamic",
			javascript: getRawJavascriptParserOptions(parser)
		};
	}
	if (type === "javascript/esm") {
		return {
			type: "javascript/esm",
			javascript: getRawJavascriptParserOptions(parser)
		};
	}
	if (type === "css") {
		return {
			type: "css",
			css: getRawCssParserOptions(parser)
		};
	}
	if (type === "css/auto") {
		return {
			type: "css/auto",
			cssAuto: getRawCssParserOptions(parser)
		};
	}
	if (type === "css/module") {
		return {
			type: "css/module",
			cssModule: getRawCssParserOptions(parser)
		};
	}
	// FIXME: shouldn't depend on module type, for example: `rules: [{ test: /\.css/, generator: {..} }]` will error
	throw new Error(`unreachable: unknow module type: ${type}`);
}

function getRawJavascriptParserOptions(
	parser: JavascriptParserOptions
): RawJavascriptParserOptions {
	return {
		dynamicImportMode: parser.dynamicImportMode,
		dynamicImportPreload: parser.dynamicImportPreload?.toString(),
		dynamicImportPrefetch: parser.dynamicImportPrefetch?.toString(),
		dynamicImportFetchPriority: parser.dynamicImportFetchPriority,
		importMeta: parser.importMeta,
		url: parser.url?.toString(),
		exprContextCritical: parser.exprContextCritical,
		wrappedContextCritical: parser.wrappedContextCritical,
		wrappedContextRegExp: parser.wrappedContextRegExp,
		exportsPresence:
			parser.exportsPresence === false ? "false" : parser.exportsPresence,
		importExportsPresence:
			parser.importExportsPresence === false
				? "false"
				: parser.importExportsPresence,
		reexportExportsPresence:
			parser.reexportExportsPresence === false
				? "false"
				: parser.reexportExportsPresence,
		strictExportPresence: parser.strictExportPresence,
		worker:
			typeof parser.worker === "boolean"
				? parser.worker
					? ["..."]
					: []
				: parser.worker,
		overrideStrict: parser.overrideStrict,
		requireAsExpression: parser.requireAsExpression,
		requireDynamic: parser.requireDynamic,
		requireResolve: parser.requireResolve,
		importDynamic: parser.importDynamic
	};
}

function getRawAssetParserOptions(
	parser: AssetParserOptions
): RawAssetParserOptions {
	return {
		dataUrlCondition: parser.dataUrlCondition
			? getRawAssetParserDataUrl(parser.dataUrlCondition)
			: undefined
	};
}

function getRawAssetParserDataUrl(
	dataUrlCondition: AssetParserDataUrl
): RawAssetParserDataUrl {
	if (typeof dataUrlCondition === "object" && dataUrlCondition !== null) {
		return {
			type: "options",
			options: {
				maxSize: dataUrlCondition.maxSize
			}
		};
	}
	throw new Error(
		`unreachable: AssetParserDataUrl type should be one of "options", but got ${dataUrlCondition}`
	);
}

function getRawCssParserOptions(
	parser: CssParserOptions
): RawCssParserOptions | RawCssAutoParserOptions | RawCssModuleParserOptions {
	return {
		namedExports: parser.namedExports
	};
}

function getRawGeneratorOptions(
	generator: { [k: string]: any },
	type: string
): RawGeneratorOptions | undefined {
	if (type === "asset") {
		return {
			type: "asset",
			asset: generator ? getRawAssetGeneratorOptions(generator) : undefined
		};
	}
	if (type === "asset/inline") {
		return {
			type: "asset/inline",
			assetInline: generator
				? getRawAssetInlineGeneratorOptions(generator)
				: undefined
		};
	}
	if (type === "asset/resource") {
		return {
			type: "asset/resource",
			assetResource: generator
				? getRawAssetResourceGeneratorOptions(generator)
				: undefined
		};
	}
	if (type === "css") {
		return {
			type: "css",
			css: getRawCssGeneratorOptions(generator)
		};
	}
	if (type === "css/auto") {
		return {
			type: "css/auto",
			cssAuto: getRawCssAutoOrModuleGeneratorOptions(generator)
		};
	}
	if (type === "css/module") {
		return {
			type: "css/module",
			cssModule: getRawCssAutoOrModuleGeneratorOptions(generator)
		};
	}

	if (
		[
			"javascript",
			"javascript/auto",
			"javascript/dynamic",
			"javascript/esm"
		].includes(type)
	) {
		return undefined;
	}

	throw new Error(`unreachable: unknow module type: ${type}`);
}

function getRawAssetGeneratorOptions(
	options: AssetGeneratorOptions
): RawAssetGeneratorOptions {
	return {
		...getRawAssetInlineGeneratorOptions(options),
		...getRawAssetResourceGeneratorOptions(options)
	};
}

function getRawAssetInlineGeneratorOptions(
	options: AssetInlineGeneratorOptions
): RawAssetInlineGeneratorOptions {
	return {
		dataUrl: options.dataUrl
			? getRawAssetGeneratorDataUrl(options.dataUrl)
			: undefined
	};
}

function getRawAssetResourceGeneratorOptions(
	options: AssetResourceGeneratorOptions
): RawAssetResourceGeneratorOptions {
	return {
		emit: options.emit,
		filename: options.filename,
		outputPath: options.outputPath,
		publicPath: options.publicPath
	};
}

function getRawAssetGeneratorDataUrl(dataUrl: AssetGeneratorDataUrl) {
	if (typeof dataUrl === "object" && dataUrl !== null) {
		const encoding = dataUrl.encoding === false ? "false" : dataUrl.encoding;
		return {
			encoding,
			mimetype: dataUrl.mimetype
		} as const;
	}
	if (typeof dataUrl === "function" && dataUrl !== null) {
		return (source: Buffer, context: RawAssetGeneratorDataUrlFnCtx) => {
			return dataUrl(source, {
				...context,
				module: Module.__from_binding(context.module)
			});
		};
	}
	throw new Error(
		`unreachable: AssetGeneratorDataUrl type should be one of "options", "function", but got ${dataUrl}`
	);
}

function getRawCssGeneratorOptions(
	options: CssGeneratorOptions
): RawCssGeneratorOptions {
	return {
		exportsOnly: options.exportsOnly,
		esModule: options.esModule
	};
}

function getRawCssAutoOrModuleGeneratorOptions(
	options: CssAutoGeneratorOptions
): RawCssAutoGeneratorOptions | RawCssModuleGeneratorOptions {
	return {
		localIdentName: options.localIdentName,
		exportsConvention: options.exportsConvention,
		exportsOnly: options.exportsOnly,
		esModule: options.esModule
	};
}

function getRawOptimization(
	optimization: Optimization
): RawOptions["optimization"] {
	assert(
		!isNil(optimization.removeAvailableModules) &&
			!isNil(optimization.sideEffects) &&
			!isNil(optimization.realContentHash) &&
			!isNil(optimization.providedExports) &&
			!isNil(optimization.usedExports) &&
			!isNil(optimization.innerGraph) &&
			"optimization.moduleIds, optimization.removeAvailableModules, optimization.removeEmptyChunks, optimization.sideEffects, optimization.realContentHash, optimization.providedExports, optimization.usedExports, optimization.innerGraph, optimization.concatenateModules should not be nil after defaults"
	);
	return {
		removeAvailableModules: optimization.removeAvailableModules,
		sideEffects: String(optimization.sideEffects),
		usedExports: String(optimization.usedExports),
		providedExports: optimization.providedExports,
		innerGraph: optimization.innerGraph,
		concatenateModules: !!optimization.concatenateModules,
		mangleExports: String(optimization.mangleExports)
	};
}

function getRawSnapshotOptions(
	_snapshot: SnapshotOptions
): RawOptions["snapshot"] {
	return {};
}

function getRawExperiments(
	experiments: ExperimentsNormalized
): RawOptions["experiments"] {
	const { topLevelAwait, layers, incremental, rspackFuture, cache } =
		experiments;
	assert(
		!isNil(topLevelAwait) &&
			!isNil(rspackFuture) &&
			!isNil(layers) &&
			!isNil(incremental)
	);

	return {
		layers,
		topLevelAwait,
		cache: getRawExperimentCache(cache),
		incremental: getRawIncremental(incremental),
		rspackFuture: getRawRspackFutureOptions(rspackFuture)
	};
}

function getRawExperimentCache(
	cache?: ExperimentCacheNormalized
): RawExperiments["cache"] {
	if (cache === undefined) {
		throw new Error("experiment cache can not be undefined");
	}
	if (typeof cache === "boolean") {
		return {
			type: cache ? "memory" : "disable"
		};
	}
	return cache;
}

function getRawIncremental(
	incremental: false | Incremental
): RawIncremental | undefined {
	if (incremental === false) {
		return undefined;
	}
	return {
		make: incremental.make!,
		inferAsyncModules: incremental.inferAsyncModules!,
		providedExports: incremental.providedExports!,
		dependenciesDiagnostics: incremental.dependenciesDiagnostics!,
		buildChunkGraph: incremental.buildChunkGraph!,
		moduleIds: incremental.moduleIds!,
		chunkIds: incremental.chunkIds!,
		modulesHashes: incremental.modulesHashes!,
		modulesCodegen: incremental.modulesCodegen!,
		modulesRuntimeRequirements: incremental.modulesRuntimeRequirements!,
		chunksRuntimeRequirements: incremental.chunksRuntimeRequirements!,
		chunksHashes: incremental.chunksHashes!,
		chunksRender: incremental.chunksRender!,
		emitAssets: incremental.emitAssets!
	};
}

function getRawRspackFutureOptions(
	future: RspackFutureOptions
): RawRspackFuture {
	return {};
}

function getRawNode(node: Node): RawOptions["node"] {
	if (node === false) {
		return undefined;
	}
	assert(
		!isNil(node.__dirname) && !isNil(node.global) && !isNil(node.__filename)
	);
	return {
		dirname: String(node.__dirname),
		filename: String(node.__filename),
		global: String(node.global)
	};
}

function getRawStats(stats: StatsValue): RawOptions["stats"] {
	const statsOptions = normalizeStatsPreset(stats);
	return {
		colors: statsOptions.colors ?? false
	};
}

export function getRawChunkLoading(chunkLoading: ChunkLoading) {
	return chunkLoading === false ? "false" : chunkLoading;
}
