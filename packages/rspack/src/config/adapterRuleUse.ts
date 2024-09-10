import type {
	JsAssetInfo,
	RawModuleRuleUse,
	RawOptions
} from "@rspack/binding";
import type { ResolveRequest } from "enhanced-resolve";

import type { Compilation } from "../Compilation";
import type { Compiler } from "../Compiler";
import type { Module } from "../Module";
import { resolvePluginImport } from "../builtin-loader";
import {
	type FeatureOptions,
	toFeatures
} from "../builtin-loader/lightningcss";
import { type LoaderObject, parsePathQueryFragment } from "../loader-runner";
import type { Logger } from "../logging/Logger";
import { isNil } from "../util";
import type Hash from "../util/hash";
import type {
	Mode,
	PublicPath,
	Resolve,
	RuleSetLoaderWithOptions,
	RuleSetUseItem,
	Target
} from "./zod";

export const BUILTIN_LOADER_PREFIX = "builtin:";

export interface ComposeJsUseOptions {
	devtool: RawOptions["devtool"];
	context: RawOptions["context"];
	mode: RawOptions["mode"];
	experiments: RawOptions["experiments"];
	compiler: Compiler;
}

export interface SourceMap {
	version: number;
	sources: string[];
	mappings: string;
	file?: string;
	sourceRoot?: string;
	sourcesContent?: string[];
	names?: string[];
}

export interface AdditionalData {
	[index: string]: any;
}

export interface LoaderContext<OptionsType = {}> {
	version: 2;
	resource: string;
	resourcePath: string;
	resourceQuery: string;
	resourceFragment: string;
	async(): (
		err?: Error | null,
		content?: string | Buffer,
		sourceMap?: string | SourceMap,
		additionalData?: AdditionalData
	) => void;
	callback(
		err?: Error | null,
		content?: string | Buffer,
		sourceMap?: string | SourceMap,
		additionalData?: AdditionalData
	): void;
	cacheable(cacheable?: boolean): void;
	sourceMap: boolean;
	rootContext: string;
	context: string | null;
	loaderIndex: number;
	remainingRequest: string;
	currentRequest: string;
	previousRequest: string;
	request: string;
	/**
	 * An array of all the loaders. It is writeable in the pitch phase.
	 * loaders = [{request: string, path: string, query: string, module: function}]
	 *
	 * In the example:
	 * [
	 *   { request: "/abc/loader1.js?xyz",
	 *     path: "/abc/loader1.js",
	 *     query: "?xyz",
	 *     module: [Function]
	 *   },
	 *   { request: "/abc/node_modules/loader2/index.js",
	 *     path: "/abc/node_modules/loader2/index.js",
	 *     query: "",
	 *     module: [Function]
	 *   }
	 * ]
	 */
	loaders: LoaderObject[];
	mode?: Mode;
	target?: Target;
	hot?: boolean;
	/**
	 * @param schema To provide the best performance, Rspack does not perform the schema validation. If your loader requires schema validation, please call scheme-utils or zod on your own.
	 */
	getOptions(schema?: any): OptionsType;
	resolve(
		context: string,
		request: string,
		callback: (
			arg0: null | Error,
			arg1?: string | false,
			arg2?: ResolveRequest
		) => void
	): void;
	getResolve(
		options: Resolve
	): (context: any, request: any, callback: any) => Promise<any>;
	getLogger(name: string): Logger;
	emitError(error: Error): void;
	emitWarning(warning: Error): void;
	emitFile(
		name: string,
		content: string | Buffer,
		sourceMap?: string,
		assetInfo?: JsAssetInfo
	): void;
	addDependency(file: string): void;
	dependency(file: string): void;
	addContextDependency(context: string): void;
	addMissingDependency(missing: string): void;
	clearDependencies(): void;
	getDependencies(): string[];
	getContextDependencies(): string[];
	getMissingDependencies(): string[];
	addBuildDependency(file: string): void;
	importModule(
		request: string,
		options: { layer?: string; publicPath?: PublicPath; baseUri?: string },
		callback: (err?: Error, res?: any) => void
	): void;
	fs: any;
	utils: {
		absolutify: (context: string, request: string) => string;
		contextify: (context: string, request: string) => string;
		createHash: (algorithm?: string) => Hash;
	};
	query: string | OptionsType;
	data: unknown;
	_compiler: Compiler;
	_compilation: Compilation;
	_module: Module;
}

export type LoaderDefinitionFunction<
	OptionsType = {},
	ContextAdditions = {}
> = (
	this: LoaderContext<OptionsType> & ContextAdditions,
	content: string,
	sourceMap?: string | SourceMap,
	additionalData?: AdditionalData
) => string | void | Buffer | Promise<string | Buffer>;

export type PitchLoaderDefinitionFunction<
	OptionsType = {},
	ContextAdditions = {}
> = (
	this: LoaderContext<OptionsType> & ContextAdditions,
	remainingRequest: string,
	previousRequest: string,
	data: object
) => string | void | Buffer | Promise<string | Buffer>;

export type LoaderDefinition<
	OptionsType = {},
	ContextAdditions = {}
> = LoaderDefinitionFunction<OptionsType, ContextAdditions> & {
	raw?: false;
	pitch?: PitchLoaderDefinitionFunction;
};

export function createRawModuleRuleUses(
	uses: RuleSetUseItem | RuleSetUseItem[],
	path: string,
	options: ComposeJsUseOptions
): RawModuleRuleUse[] {
	const normalizeRuleSetUseItem = (
		item: RuleSetUseItem
	): RuleSetLoaderWithOptions =>
		typeof item === "string" ? { loader: item } : item;
	const allUses = Array.isArray(uses)
		? [...uses].map(normalizeRuleSetUseItem)
		: [normalizeRuleSetUseItem(uses)];
	return createRawModuleRuleUsesImpl(allUses, path, options);
}

type GetLoaderOptions = (
	o: RuleSetLoaderWithOptions["options"],
	options: ComposeJsUseOptions
) => RuleSetLoaderWithOptions["options"];

const getSwcLoaderOptions: GetLoaderOptions = (o, _) => {
	if (o && typeof o === "object" && o.rspackExperiments) {
		const expr = o.rspackExperiments;
		if (expr.import || expr.pluginImport) {
			expr.import = resolvePluginImport(expr.import || expr.pluginImport);
		}
	}
	return o;
};

const getLightningcssLoaderOptions: GetLoaderOptions = (o, _) => {
	if (o && typeof o === "object") {
		if (typeof o.targets === "string") {
			o.targets = [o.targets];
		}

		if (o.include && typeof o.include === "object") {
			o.include = toFeatures(o.include as unknown as FeatureOptions);
		}

		if (o.exclude && typeof o.exclude === "object") {
			o.exclude = toFeatures(o.exclude as unknown as FeatureOptions);
		}
	}

	return o;
};

function getBuiltinLoaderOptions(
	identifier: string,
	o: RuleSetLoaderWithOptions["options"],
	options: ComposeJsUseOptions
): RuleSetLoaderWithOptions["options"] {
	if (identifier.startsWith(`${BUILTIN_LOADER_PREFIX}swc-loader`)) {
		return getSwcLoaderOptions(o, options);
	}

	if (identifier.startsWith(`${BUILTIN_LOADER_PREFIX}lightningcss-loader`)) {
		return getLightningcssLoaderOptions(o, options);
	}

	return o;
}

function createRawModuleRuleUsesImpl(
	uses: RuleSetLoaderWithOptions[],
	path: string,
	options: ComposeJsUseOptions
): RawModuleRuleUse[] {
	if (!uses.length) {
		return [];
	}

	return uses.map((use, index) => {
		let o: string | undefined;
		let isBuiltin = false;
		if (use.loader.startsWith(BUILTIN_LOADER_PREFIX)) {
			const temp = getBuiltinLoaderOptions(use.loader, use.options, options);
			// keep json with indent so miette can show pretty error
			o = isNil(temp)
				? undefined
				: typeof temp === "string"
					? temp
					: JSON.stringify(temp, null, 2);
			isBuiltin = true;
		}

		return {
			loader: resolveStringifyLoaders(
				use,
				`${path}[${index}]`,
				options.compiler,
				isBuiltin
			),
			options: o
		};
	});
}

function resolveStringifyLoaders(
	use: RuleSetLoaderWithOptions,
	path: string,
	compiler: Compiler,
	isBuiltin: boolean
) {
	const obj = parsePathQueryFragment(use.loader);
	let ident: string | null = null;

	if (use.options === null) {
	} else if (use.options === undefined) {
	} else if (typeof use.options === "string") obj.query = `?${use.options}`;
	else if (use.ident) obj.query = `??${(ident = use.ident)}`;
	else if (typeof use.options === "object" && use.options.ident)
		obj.query = `??${(ident = use.options.ident)}`;
	else if (typeof use.options === "object") obj.query = `??${(ident = path)}`;
	else obj.query = `?${JSON.stringify(use.options)}`;

	if (use.options && typeof use.options === "object") {
		if (!ident) ident = "[[missing ident]]";
		compiler.__internal__ruleSet.references.set(ident, use.options);
		if (isBuiltin) {
			compiler.__internal__ruleSet.builtinReferences.set(ident, use.options);
		}
	}

	return obj.path + obj.query + obj.fragment;
}

export function isUseSourceMap(devtool: RawOptions["devtool"]): boolean {
	return (
		devtool.includes("source-map") &&
		(devtool.includes("module") || !devtool.includes("cheap"))
	);
}

export function isUseSimpleSourceMap(devtool: RawOptions["devtool"]): boolean {
	return devtool.includes("source-map") && !isUseSourceMap(devtool);
}
