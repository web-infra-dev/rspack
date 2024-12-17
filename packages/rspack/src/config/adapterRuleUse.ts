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
} from "./types";
import { RspackOptionsNormalized } from "./normalization";

export const BUILTIN_LOADER_PREFIX = "builtin:";

export interface ComposeJsUseOptions {
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

export type LoaderContextCallback = (
	err?: Error | null,
	content?: string | Buffer,
	sourceMap?: string | SourceMap,
	additionalData?: AdditionalData
) => void;

export type ErrorWithDetails = Error & { details?: string };

// aligned with https://github.com/webpack/webpack/blob/64e8e33151c3fabd3f1917851193e458a526e803/declarations/LoaderContext.d.ts#L19
export type ResolveCallback = (
	err: null | ErrorWithDetails,
	res?: string | false,
	req?: ResolveRequest
) => void;

export interface DiagnosticLocation {
	/** Text for highlighting the location */
	text?: string;
	/** 1-based line */
	line: number;
	/** 0-based column in bytes */
	column: number;
	/** Length in bytes */
	length: number;
}

export interface Diagnostic {
	message: string;
	help?: string;
	sourceCode?: string;
	/**
	 * Location to the source code.
	 *
	 * If `sourceCode` is not provided, location will be omitted.
	 */
	location?: DiagnosticLocation;
	file?: string;
	severity: "error" | "warning";
}

interface LoaderExperiments {
	emitDiagnostic(diagnostic: Diagnostic): void;
}

export interface LoaderContext<OptionsType = {}> {
	version: 2;
	resource: string;
	resourcePath: string;
	resourceQuery: string;
	resourceFragment: string;
	async(): LoaderContextCallback;
	callback: LoaderContextCallback;
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
	):
		| ((context: string, request: string, callback: ResolveCallback) => void)
		| ((
				context: string,
				request: string
		  ) => Promise<string | false | undefined>);
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
	/**
	 * This is an experimental API and maybe subject to change.
	 * @experimental
	 */
	experiments: LoaderExperiments;
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

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 * Store some data from loader, and consume it from parser, it may be removed in the future
	 *
	 * @internal
	 */
	__internal__parseMeta: Record<string, string>;
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

const getSwcLoaderOptions: GetLoaderOptions = (options, _) => {
	if (options && typeof options === "object") {
		// enable `disableAllLints` by default to reduce performance overhead
		options.jsc ??= {};
		options.jsc.experimental ??= {};
		options.jsc.experimental.disableAllLints ??= true;

		// resolve `rspackExperiments.import` options
		const { rspackExperiments } = options;
		if (rspackExperiments) {
			if (rspackExperiments.import || rspackExperiments.pluginImport) {
				rspackExperiments.import = resolvePluginImport(
					rspackExperiments.import || rspackExperiments.pluginImport
				);
			}
		}
	}
	return options;
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

export function isUseSourceMap(
	devtool: RspackOptionsNormalized["devtool"]
): boolean {
	if (!devtool) {
		return false;
	}
	return (
		devtool.includes("source-map") &&
		(devtool.includes("module") || !devtool.includes("cheap"))
	);
}

export function isUseSimpleSourceMap(
	devtool: RspackOptionsNormalized["devtool"]
): boolean {
	if (!devtool) {
		return false;
	}
	return devtool.includes("source-map") && !isUseSourceMap(devtool);
}
