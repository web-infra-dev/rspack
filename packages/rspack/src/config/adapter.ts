import type {
	RawCacheGroupOptions,
	RawModuleRule,
	RawOptions,
	RawRuleSetCondition,
	RawRuleSetLogicalConditions,
	RawGeneratorOptions,
	RawAssetGeneratorOptions,
	RawParserOptions,
	RawAssetParserOptions,
	RawAssetParserDataUrl,
	RawAssetGeneratorDataUrl,
	RawAssetInlineGeneratorOptions,
	RawAssetResourceGeneratorOptions,
	RawIncrementalRebuild,
	RawModuleRuleUses,
	RawFuncUseCtx,
	RawRspackFuture
} from "@rspack/binding";
import assert from "assert";
import { Compiler } from "../Compiler";
import { normalizeStatsPreset } from "../Stats";
import { deprecatedWarn, isNil } from "../util";
import { parseResource } from "../util/identifier";
import {
	ComposeJsUseOptions,
	LoaderContext,
	createRawModuleRuleUses,
	LoaderDefinition,
	LoaderDefinitionFunction
} from "./adapterRuleUse";
import {
	CrossOriginLoading,
	LibraryOptions,
	Node,
	Optimization,
	Resolve,
	RuleSetCondition,
	RuleSetLogicalConditions,
	RuleSetRule,
	SnapshotOptions,
	StatsValue,
	Target,
	AssetGeneratorDataUrl,
	AssetGeneratorOptions,
	AssetInlineGeneratorOptions,
	AssetResourceGeneratorOptions,
	AssetParserDataUrl,
	AssetParserOptions,
	ParserOptionsByModuleType,
	GeneratorOptionsByModuleType,
	IncrementalRebuildOptions,
	OptimizationSplitChunksOptions,
	RspackFutureOptions,
	JavascriptParserOptions
} from "./zod";
import {
	ExperimentsNormalized,
	ModuleOptionsNormalized,
	OutputNormalized,
	RspackOptionsNormalized
} from "./normalization";

export type { LoaderContext, LoaderDefinition, LoaderDefinitionFunction };

export const getRawOptions = (
	options: RspackOptionsNormalized,
	compiler: Compiler,
	processResource: (
		loaderContext: LoaderContext,
		resourcePath: string,
		callback: any
	) => void
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
		devServer: {
			hot: options.devServer?.hot ?? false
		},
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
		profile: options.profile!,
		// TODO: remove this
		builtins: options.builtins as any
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

function getRawAlias(
	alias: Resolve["alias"] = {}
): RawOptions["resolve"]["alias"] {
	const entires = Object.entries(alias).map(([key, value]) => {
		if (Array.isArray(value)) {
			return [key, value];
		} else {
			return [key, [value]];
		}
	});
	return Object.fromEntries(entires);
}

function getRawResolveByDependency(
	byDependency: Resolve["byDependency"]
): RawOptions["resolve"]["byDependency"] {
	if (byDependency === undefined) return byDependency;
	return Object.fromEntries(
		Object.entries(byDependency).map(([k, v]) => [k, getRawResolve(v)])
	);
}

function getRawResolve(resolve: Resolve): RawOptions["resolve"] {
	let references = resolve.tsConfig?.references;
	let tsconfigConfigFile = resolve.tsConfigPath ?? resolve.tsConfig?.configFile;
	return {
		...resolve,
		alias: getRawAlias(resolve.alias),
		fallback: getRawAlias(resolve.fallback),
		extensionAlias: getRawAlias(resolve.extensionAlias) as Record<
			string,
			Array<string>
		>,
		tsconfig: tsconfigConfigFile
			? {
					configFile: tsconfigConfigFile,
					referencesType:
						references == "auto" ? "auto" : references ? "manual" : "disabled",
					references: references == "auto" ? undefined : references
			  }
			: undefined,
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
		publicPath: output.publicPath!,
		clean: output.clean!,
		assetModuleFilename: output.assetModuleFilename!,
		filename: output.filename!,
		chunkFilename: output.chunkFilename!,
		chunkLoading: chunkLoading === false ? "false" : chunkLoading,
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
		workerPublicPath: output.workerPublicPath!
	};
}

function getRawLibrary(
	library: LibraryOptions
): RawOptions["output"]["library"] {
	const {
		type,
		name,
		export: libraryExport,
		umdNamedDefine,
		auxiliaryComment
	} = library;
	return {
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
		name:
			name == null
				? name
				: typeof name === "object" && !Array.isArray(name)
				? {
						amd: name.amd,
						commonjs: name.commonjs,
						root:
							Array.isArray(name.root) || name.root == null
								? name.root
								: [name.root]
				  }
				: {
						amd: Array.isArray(name) ? name[0] : name,
						commonjs: Array.isArray(name) ? name[0] : name,
						root: Array.isArray(name) || name == null ? name : [name]
				  },
		export:
			Array.isArray(libraryExport) || libraryExport == null
				? libraryExport
				: [libraryExport],
		umdNamedDefine
	};
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
		getRawModuleRule(rule, `ruleSet[${index}]`, options)
	);
	return {
		rules,
		parser: getRawParserOptionsByModuleType(module.parser),
		generator: getRawGeneratorOptionsByModuleType(module.generator)
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
			return condition.not.every(c => !tryMatch(payload, c));
		}
	}

	return false;
}

const deprecatedRuleType = (type?: string) => {
	type ??= "javascript/auto";
	if (/ts|typescript|tsx|typescriptx|jsx|javascriptx/.test(type)) {
		deprecatedWarn(
			`'Rule.type: ${type}' has been deprecated, please migrate to builtin:swc-loader with type 'javascript/auto'`
		);
	}
};

const getRawModuleRule = (
	rule: RuleSetRule,
	path: string,
	options: ComposeJsUseOptions
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
	let funcUse;
	if (typeof rule.use === "function") {
		funcUse = (rawContext: RawFuncUseCtx) => {
			const context = {
				...rawContext,
				compiler: options.compiler
			};
			const uses = (
				rule.use as Exclude<RawModuleRuleUses["funcUse"], undefined>
			)(context);

			return createRawModuleRuleUses(uses ?? [], `${path}.use`, options);
		};
	}

	let rawModuleRule: RawModuleRule = {
		test: rule.test ? getRawRuleSetCondition(rule.test) : undefined,
		include: rule.include ? getRawRuleSetCondition(rule.include) : undefined,
		exclude: rule.exclude ? getRawRuleSetCondition(rule.exclude) : undefined,
		issuer: rule.issuer ? getRawRuleSetCondition(rule.issuer) : undefined,
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
				? { type: "function", funcUse }
				: {
						type: "array",
						arrayUse: createRawModuleRuleUses(
							rule.use ?? [],
							`${path}.use`,
							options
						)
				  },
		type: rule.type,
		parser: rule.parser
			? getRawParserOptions(rule.parser, rule.type ?? "javascript/auto")
			: undefined,
		generator: rule.generator
			? getRawGeneratorOptions(rule.generator, rule.type ?? "javascript/auto")
			: undefined,
		resolve: rule.resolve ? getRawResolve(rule.resolve) : undefined,
		oneOf: rule.oneOf
			? rule.oneOf.map((rule, index) =>
					getRawModuleRule(rule, `${path}.oneOf[${index}]`, options)
			  )
			: undefined,
		rules: rule.rules
			? rule.rules.map((rule, index) =>
					getRawModuleRule(rule, `${path}.rules[${index}]`, options)
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

		rawModuleRule.rspackResource = getRawRuleSetCondition(function (
			resourceQueryFragment
		) {
			const { path, query, fragment } = parseResource(resourceQueryFragment);

			if (rule.test && !tryMatch(path, rule.test)) {
				return false;
			} else if (rule.resource && !tryMatch(path, rule.resource)) {
				return false;
			}

			if (rule.resourceQuery && !tryMatch(query, rule.resourceQuery)) {
				return false;
			}

			if (rule.resourceFragment && !tryMatch(fragment, rule.resourceFragment)) {
				return false;
			}

			return true;
		});
	}
	if (options.experiments.rspackFuture.disableTransformByDefault) {
		deprecatedRuleType(rule.type);
	}
	return rawModuleRule;
};

function getRawRuleSetCondition(
	condition: RuleSetCondition
): RawRuleSetCondition {
	if (typeof condition === "string") {
		return {
			type: "string",
			stringMatcher: condition
		};
	}
	if (condition instanceof RegExp) {
		return {
			type: "regexp",
			regexpMatcher: condition.source
		};
	}
	if (typeof condition === "function") {
		return {
			type: "function",
			funcMatcher: condition
		};
	}
	if (Array.isArray(condition)) {
		return {
			type: "array",
			arrayMatcher: condition.map(i => getRawRuleSetCondition(i))
		};
	}
	if (typeof condition === "object" && condition !== null) {
		return {
			type: "logical",
			logicalMatcher: [getRawRuleSetLogicalConditions(condition)]
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

function getRawParserOptionsByModuleType(
	parser: ParserOptionsByModuleType
): Record<string, RawParserOptions> {
	return Object.fromEntries(
		Object.entries(parser).map(([k, v]) => [k, getRawParserOptions(v, k)])
	);
}

function getRawGeneratorOptionsByModuleType(
	parser: GeneratorOptionsByModuleType
): Record<string, RawGeneratorOptions> {
	return Object.fromEntries(
		Object.entries(parser).map(([k, v]) => [k, getRawGeneratorOptions(v, k)])
	);
}

function getRawParserOptions(
	parser: { [k: string]: any },
	type: string
): RawParserOptions {
	if (type === "asset") {
		return {
			type: "asset",
			asset: getRawAssetParserOptions(parser)
		};
	} else if (type === "javascript") {
		return {
			type: "javascript",
			javascript: getRawJavascriptParserOptions(parser)
		};
	}
	return {
		type: "unknown"
	};
}

function getRawJavascriptParserOptions(parser: JavascriptParserOptions) {
	return {
		dynamicImportMode: parser.dynamicImportMode ?? "lazy"
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

function getRawGeneratorOptions(
	generator: { [k: string]: any },
	type: string
): RawGeneratorOptions {
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
	return {
		type: "unknown"
	};
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
			? getRawAssetGeneratorDaraUrl(options.dataUrl)
			: undefined
	};
}

function getRawAssetResourceGeneratorOptions(
	options: AssetResourceGeneratorOptions
): RawAssetResourceGeneratorOptions {
	return {
		filename: options.filename,
		publicPath: options.publicPath
	};
}

function getRawAssetGeneratorDaraUrl(
	dataUrl: AssetGeneratorDataUrl
): RawAssetGeneratorDataUrl {
	if (typeof dataUrl === "object" && dataUrl !== null) {
		return {
			type: "options",
			options: {
				encoding: dataUrl.encoding === false ? "false" : dataUrl.encoding,
				mimetype: dataUrl.mimetype
			}
		};
	}
	throw new Error(
		`unreachable: AssetGeneratorDataUrl type should be one of "options", but got ${dataUrl}`
	);
}

function getRawOptimization(
	optimization: Optimization
): RawOptions["optimization"] {
	assert(
		!isNil(optimization.moduleIds) &&
			!isNil(optimization.chunkIds) &&
			!isNil(optimization.removeAvailableModules) &&
			!isNil(optimization.removeEmptyChunks) &&
			!isNil(optimization.sideEffects) &&
			!isNil(optimization.realContentHash) &&
			!isNil(optimization.providedExports) &&
			!isNil(optimization.usedExports) &&
			!isNil(optimization.innerGraph),
		"optimization.moduleIds, optimization.removeAvailableModules, optimization.removeEmptyChunks, optimization.sideEffects, optimization.realContentHash, optimization.providedExports, optimization.usedExports, optimization.innerGraph should not be nil after defaults"
	);
	return {
		chunkIds: optimization.chunkIds,
		splitChunks: toRawSplitChunksOptions(optimization.splitChunks),
		moduleIds: optimization.moduleIds,
		removeAvailableModules: optimization.removeAvailableModules,
		removeEmptyChunks: optimization.removeEmptyChunks,
		sideEffects: String(optimization.sideEffects),
		realContentHash: optimization.realContentHash,
		usedExports: String(optimization.usedExports),
		providedExports: optimization.providedExports,
		innerGraph: optimization.innerGraph
	};
}

function toRawSplitChunksOptions(
	sc?: OptimizationSplitChunksOptions
): RawOptions["optimization"]["splitChunks"] | undefined {
	if (!sc) {
		return;
	}

	const { name, cacheGroups = {}, ...passThrough } = sc;
	return {
		name: name === false ? undefined : name,
		cacheGroups: Object.entries(cacheGroups)
			.filter(([_key, group]) => group !== false)
			.map(([key, group]) => {
				group = group as Exclude<typeof group, false>;

				const { test, name, ...passThrough } = group;
				const rawGroup: RawCacheGroupOptions = {
					key,
					test,
					name: name === false ? undefined : name,
					...passThrough
				};
				return rawGroup;
			}),
		...passThrough
	};
}

function getRawSnapshotOptions(
	snapshot: SnapshotOptions
): RawOptions["snapshot"] {
	const { resolve, module } = snapshot;
	assert(!isNil(resolve) && !isNil(module));
	const { timestamp: resolveTimestamp, hash: resolveHash } = resolve;
	const { timestamp: moduleTimestamp, hash: moduleHash } = module;
	assert(
		!isNil(resolveTimestamp) &&
			!isNil(resolveHash) &&
			!isNil(moduleTimestamp) &&
			!isNil(moduleHash)
	);
	return {
		resolve: {
			timestamp: resolveTimestamp,
			hash: resolveHash
		},
		module: {
			timestamp: moduleTimestamp,
			hash: moduleHash
		}
	};
}

function getRawExperiments(
	experiments: ExperimentsNormalized
): RawOptions["experiments"] {
	const {
		lazyCompilation,
		incrementalRebuild,
		asyncWebAssembly,
		newSplitChunks,
		topLevelAwait,
		css,
		rspackFuture
	} = experiments;
	assert(
		!isNil(lazyCompilation) &&
			!isNil(incrementalRebuild) &&
			!isNil(asyncWebAssembly) &&
			!isNil(newSplitChunks) &&
			!isNil(topLevelAwait) &&
			!isNil(css) &&
			!isNil(rspackFuture)
	);

	return {
		lazyCompilation,
		incrementalRebuild: getRawIncrementalRebuild(incrementalRebuild),
		asyncWebAssembly,
		newSplitChunks,
		topLevelAwait,
		css,
		rspackFuture: getRawRspackFutureOptions(rspackFuture)
	};
}

function getRawRspackFutureOptions(
	future: RspackFutureOptions
): RawRspackFuture {
	assert(!isNil(future.newResolver));
	assert(!isNil(future.newTreeshaking));
	assert(!isNil(future.disableTransformByDefault));
	return {
		newResolver: future.newResolver,
		newTreeshaking: future.newTreeshaking,
		disableTransformByDefault: future.disableTransformByDefault
	};
}

function getRawIncrementalRebuild(
	inc: false | IncrementalRebuildOptions
): RawIncrementalRebuild {
	if (inc === false) {
		return {
			make: false,
			emitAsset: false
		};
	}
	const { make, emitAsset } = inc;
	assert(!isNil(make) && !isNil(emitAsset));
	return {
		make,
		emitAsset
	};
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
