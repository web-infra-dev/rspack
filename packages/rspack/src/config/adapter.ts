import {
	RawCacheGroupOptions,
	RawExternalItem,
	RawExternalItemValue,
	RawModuleRule,
	RawOptions,
	RawRuleSetCondition,
	RawRuleSetLogicalConditions,
	RawBannerConditions,
	RawBannerCondition
} from "@rspack/binding";
import assert from "assert";
import { Compiler } from "../compiler";
import { normalizeStatsPreset } from "../stats";
import { isNil } from "../util";
import {
	ComposeJsUseOptions,
	LoaderContext,
	createRawModuleRuleUses
} from "./adapter-rule-use";
import {
	BannerConditions,
	BannerCondition,
	CrossOriginLoading,
	EntryNormalized,
	Experiments,
	ExternalItem,
	ExternalItemValue,
	Externals,
	ExternalsPresets,
	LibraryOptions,
	ModuleOptionsNormalized,
	Node,
	Optimization,
	OptimizationSplitChunksOptions,
	OutputNormalized,
	Resolve,
	RspackOptionsNormalized,
	RuleSetCondition,
	RuleSetLogicalConditions,
	RuleSetRule,
	SnapshotOptions,
	StatsValue,
	Target
} from "./types";

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
	let rawEntry = getRawEntry(options.entry);
	return {
		entry: rawEntry,
		entryOrder: Object.keys(rawEntry),
		mode: options.mode,
		target: getRawTarget(options.target),
		context: options.context,
		output: getRawOutput(options.output),
		resolve: getRawResolve(options.resolve),
		module: getRawModule(options.module, {
			compiler,
			devtool,
			context: options.context
		}),
		externals: options.externals
			? getRawExternals(options.externals)
			: undefined,
		externalsType:
			options.externalsType === undefined ? "" : options.externalsType,
		externalsPresets: getRawExternalsPresets(options.externalsPresets),
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
		experiments: getRawExperiments(options.experiments),
		node: getRawNode(options.node),
		// TODO: refactor builtins
		builtins: options.builtins as any
	};
};

function getRawEntry(entry: EntryNormalized): RawOptions["entry"] {
	const raw: RawOptions["entry"] = {};
	for (const key of Object.keys(entry)) {
		const runtime = entry[key].runtime;
		raw[key] = {
			import: entry[key].import!,
			runtime: runtime === false ? undefined : runtime
		};
	}
	return raw;
}

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
	return {
		...resolve,
		alias: getRawAlias(resolve.alias),
		fallback: getRawAlias(resolve.fallback),
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
	const wasmLoading = output.wasmLoading!;
	return {
		path: output.path!,
		publicPath: output.publicPath!,
		clean: output.clean!,
		assetModuleFilename: output.assetModuleFilename!,
		filename: output.filename!,
		chunkFormat: output.chunkFormat === false ? undefined : output.chunkFormat!,
		chunkFilename: output.chunkFilename!,
		chunkLoading:
			output.chunkLoading === false ? undefined : output.chunkLoading!,
		crossOriginLoading: getRawCrossOriginLoading(output.crossOriginLoading!),
		cssFilename: output.cssFilename!,
		cssChunkFilename: output.cssChunkFilename!,
		hotUpdateChunkFilename: output.hotUpdateChunkFilename!,
		hotUpdateMainFilename: output.hotUpdateMainFilename!,
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
		sourceMapFilename: output.sourceMapFilename!
	};
}

function getRawExternalsPresets(
	presets: ExternalsPresets
): RawOptions["externalsPresets"] {
	return {
		web: presets.web ?? false,
		node: presets.node ?? false
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
	// TODO: workaround for module.defaultRules
	const rules = (
		[...module.defaultRules, ...module.rules] as RuleSetRule[]
	).map<RawModuleRule>(i => getRawModuleRule(i, options));
	return {
		rules,
		parser: module.parser
	};
}

const getRawModuleRule = (
	rule: RuleSetRule,
	options: ComposeJsUseOptions
): RawModuleRule => {
	return {
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
		sideEffects: rule.sideEffects,
		use: createRawModuleRuleUses(rule.use ?? [], options),
		type: rule.type,
		parser: rule.parser,
		generator: rule.generator,
		resolve: rule.resolve ? getRawResolve(rule.resolve) : undefined,
		oneOf: rule.oneOf
			? rule.oneOf.map(i => getRawModuleRule(i, options))
			: undefined,
		enforce: rule.enforce
	};
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

export function getBannerCondition(
	condition: BannerCondition
): RawBannerCondition {
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
	throw new Error("unreachable: condition should be one of string, RegExp");
}

export function getBannerConditions(
	condition?: BannerConditions
): RawBannerConditions | undefined {
	if (!condition) return undefined;

	if (Array.isArray(condition)) {
		return {
			type: "array",
			arrayMatcher: condition.map(i => getBannerCondition(i))
		};
	}

	return getBannerCondition(condition);
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

function getRawExternals(externals: Externals): RawOptions["externals"] {
	function getRawExternalItem(item: ExternalItem): RawExternalItem {
		if (typeof item === "string") {
			return { type: "string", stringPayload: item };
		} else if (item instanceof RegExp) {
			return { type: "regexp", regexpPayload: item.source };
		} else if (typeof item === "function") {
			return {
				type: "function",
				fnPayload: async ctx => {
					return await new Promise((resolve, reject) => {
						const promise = item(ctx, (err, result, type) => {
							if (err) reject(err);
							resolve({
								result: getRawExternalItemValueFormFnResult(result),
								external_type: type
							});
						});
						if (promise && promise.then) {
							promise.then(
								result =>
									resolve({
										result: getRawExternalItemValueFormFnResult(result),
										external_type: undefined
									}),
								e => reject(e)
							);
						}
					});
				}
			};
		}
		return {
			type: "object",
			objectPayload: Object.fromEntries(
				Object.entries(item).map(([k, v]) => [k, getRawExternalItemValue(v)])
			)
		};
	}
	function getRawExternalItemValueFormFnResult(result?: ExternalItemValue) {
		return result === undefined ? result : getRawExternalItemValue(result);
	}
	function getRawExternalItemValue(
		value: ExternalItemValue
	): RawExternalItemValue {
		if (typeof value === "string") {
			return { type: "string", stringPayload: value };
		} else if (typeof value === "boolean") {
			return { type: "bool", boolPayload: value };
		} else if (Array.isArray(value)) {
			return {
				type: "array",
				arrayPayload: value
			};
		}
		throw new Error("unreachable");
	}

	if (Array.isArray(externals)) {
		return externals.map(i => getRawExternalItem(i));
	}
	return [getRawExternalItem(externals)];
}

function getRawOptimization(
	optimization: Optimization
): RawOptions["optimization"] {
	assert(
		!isNil(optimization.moduleIds) &&
			!isNil(optimization.removeAvailableModules) &&
			!isNil(optimization.removeEmptyChunks) &&
			!isNil(optimization.sideEffects) &&
			!isNil(optimization.realContentHash),
		"optimization.moduleIds, optimization.removeAvailableModules, optimization.removeEmptyChunks, optimization.sideEffects, optimization.realContentHash should not be nil after defaults"
	);
	return {
		splitChunks: optimization.splitChunks
			? getRawSplitChunksOptions(optimization.splitChunks)
			: undefined,
		moduleIds: optimization.moduleIds,
		removeAvailableModules: optimization.removeAvailableModules,
		removeEmptyChunks: optimization.removeEmptyChunks,
		sideEffects: String(optimization.sideEffects),
		realContentHash: optimization.realContentHash
	};
}

function getRawSplitChunksOptions(
	sc: OptimizationSplitChunksOptions
): RawOptions["optimization"]["splitChunks"] {
	return {
		name: sc.name === false ? undefined : sc.name,
		cacheGroups: sc.cacheGroups
			? Object.fromEntries(
					Object.entries(sc.cacheGroups).map(([key, group]) => {
						let normalizedGroup: RawCacheGroupOptions = {
							test: group.test ? group.test.source : undefined,
							name: group.name === false ? undefined : group.name,
							priority: group.priority,
							minChunks: group.minChunks,
							chunks: group.chunks,
							reuseExistingChunk: group.reuseExistingChunk,
							minSize: group.minSize,
							maxAsyncSize: group.maxAsyncSize,
							maxInitialSize: group.maxInitialSize,
							maxSize: group.maxSize
						};
						return [key, normalizedGroup];
					})
			  )
			: {},
		chunks: sc.chunks,
		maxAsyncRequests: sc.maxAsyncRequests,
		maxInitialRequests: sc.maxInitialRequests,
		minChunks: sc.minChunks,
		minSize: sc.minSize,
		enforceSizeThreshold: sc.enforceSizeThreshold,
		minRemainingSize: sc.minRemainingSize,
		maxSize: sc.maxSize,
		maxAsyncSize: sc.maxAsyncSize,
		maxInitialSize: sc.maxInitialSize,
		fallbackCacheGroup: sc.fallbackCacheGroup
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
	experiments: Experiments
): RawOptions["experiments"] {
	const {
		lazyCompilation,
		incrementalRebuild,
		asyncWebAssembly,
		newSplitChunks,
		css
	} = experiments;
	assert(
		!isNil(lazyCompilation) &&
			!isNil(incrementalRebuild) &&
			!isNil(asyncWebAssembly) &&
			!isNil(newSplitChunks) &&
			!isNil(css)
	);
	return {
		lazyCompilation,
		incrementalRebuild,
		asyncWebAssembly,
		newSplitChunks,
		css
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
