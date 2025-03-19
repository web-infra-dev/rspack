import assert from "node:assert";
import {
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
	type RawEnvironment,
	type RawFuncUseCtx,
	type RawGeneratorOptions,
	type RawJavascriptParserOptions,
	type RawJsonGeneratorOptions,
	type RawJsonParserOptions,
	type RawModuleRule,
	type RawModuleRuleUse,
	type RawOptions,
	type RawOutputOptions,
	type RawParserOptions,
	type RawRuleSetCondition,
	RawRuleSetConditionType,
	type RawRuleSetLogicalConditions
} from "@rspack/binding";

import type { Compiler } from "../Compiler";
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
	CssAutoGeneratorOptions,
	CssGeneratorOptions,
	CssParserOptions,
	GeneratorOptionsByModuleType,
	JavascriptParserOptions,
	JsonGeneratorOptions,
	JsonParserOptions,
	Node,
	Optimization,
	Output,
	ParserOptionsByModuleType,
	Resolve,
	RuleSetCondition,
	RuleSetLogicalConditions,
	RuleSetRule,
	StatsValue
} from "./types";

export type { LoaderContext, LoaderDefinition, LoaderDefinitionFunction };

// invariant: `options` is normalized with default value applied
export const getRawOptions = (
	options: RspackOptionsNormalized,
	compiler: Compiler
): RawOptions => {
	const mode = options.mode;
	const experiments = options.experiments as Required<ExperimentsNormalized>;
	return {
		name: options.name,
		mode,
		context: options.context!,
		output: getRawOutput(options.output),
		resolve: getRawResolve(options.resolve),
		resolveLoader: getRawResolve(options.resolveLoader),
		module: getRawModule(options.module, {
			compiler,
			mode,
			context: options.context!,
			experiments
		}),
		optimization: options.optimization as Required<Optimization>,
		stats: getRawStats(options.stats),
		cache: {
			type: options.cache ? "memory" : "disable"
		},
		experiments,
		node: getRawNode(options.node),
		profile: options.profile!,
		amd: options.amd ? JSON.stringify(options.amd || {}) : undefined,
		bail: options.bail!,
		__references: {}
	};
};

function getRawOutput(output: Output): RawOutputOptions {
	return {
		...(output as Required<OutputNormalized>),
		environment: getRawOutputEnvironment(output.environment)
	};
}

function getRawOutputEnvironment(
	environment: Output["environment"] = {}
): RawEnvironment {
	return {
		const: Boolean(environment.const),
		arrowFunction: Boolean(environment.arrowFunction),
		nodePrefixForCoreModules: Boolean(environment.nodePrefixForCoreModules),
		asyncFunction: Boolean(environment.asyncFunction),
		bigIntLiteral: Boolean(environment.bigIntLiteral),
		destructuring: Boolean(environment.destructuring),
		document: Boolean(environment.document),
		dynamicImport: Boolean(environment.dynamicImport),
		forOf: Boolean(environment.forOf),
		globalThis: Boolean(environment.globalThis),
		module: Boolean(environment.module),
		optionalChaining: Boolean(environment.optionalChaining),
		templateLiteral: Boolean(environment.templateLiteral)
	};
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

	if (type === "json") {
		return {
			type: "json",
			json: getRawJsonParserOptions(parser)
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

function getRawJsonParserOptions(
	parser: JsonParserOptions
): RawJsonParserOptions {
	return {
		exportsDepth: parser.exportsDepth,
		parse:
			typeof parser.parse === "function"
				? str => JSON.stringify(parser.parse!(str))
				: undefined
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
	if (type === "json") {
		return {
			type: "json",
			json: getRawJsonGeneratorOptions(generator)
		};
	}

	if (
		[
			"asset/source",
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
		publicPath: options.publicPath,
		importMode: options.importMode
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
			return dataUrl(source, context);
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

function getRawJsonGeneratorOptions(
	options: JsonGeneratorOptions
): RawJsonGeneratorOptions {
	return {
		JSONParse: options.JSONParse
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
