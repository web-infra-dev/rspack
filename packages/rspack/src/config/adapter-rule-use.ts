import { JsAssetInfo, RawModuleRuleUse, RawOptions } from "@rspack/binding";
import assert from "assert";
import { ResolveRequest } from "enhanced-resolve";

import { Compiler } from "../compiler";
import { Logger } from "../logging/Logger";
import Hash from "../util/hash";
import {
	Mode,
	Resolve,
	RuleSetUse,
	RuleSetUseItem,
	RuleSetLoaderWithOptions
} from "./types";
import { parsePathQueryFragment } from "../loader-runner";

const BUILTIN_LOADER_PREFIX = "builtin:";

export interface ComposeJsUseOptions {
	devtool: RawOptions["devtool"];
	context: RawOptions["context"];
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

export interface LoaderObject {
	request: string;
	path: string;
	query: string;
	fragment: string;
	options: object | string | undefined;
	ident: string;
	normal: Function | undefined;
	pitch: Function | undefined;
	raw: boolean | undefined;
	data: object | undefined;
	pitchExecuted: boolean;
	normalExecuted: boolean;
}

export interface LoaderContext {
	version: 2;
	resource: string;
	resourcePath: string;
	resourceQuery: string;
	resourceFragment: string;
	async(): (
		err: Error | null,
		content: string | Buffer,
		sourceMap?: string | SourceMap,
		additionalData?: AdditionalData
	) => void;
	callback(
		err: Error | null,
		content: string | Buffer,
		sourceMap?: string | SourceMap,
		additionalData?: AdditionalData
	): void;
	cacheable(cacheable?: boolean): void;
	sourceMap: boolean;
	rootContext: string;
	context: string;
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
	hot?: boolean;
	getOptions(schema?: any): unknown;
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
	fs: any;
	utils: {
		absolutify: (context: string, request: string) => string;
		contextify: (context: string, request: string) => string;
		createHash: (algorithm?: string) => Hash;
	};
	query: unknown;
	data: unknown;
	_compiler: Compiler;
	_compilation: Compiler["compilation"];
	/**
	 * Internal field for interoperability.
	 * Do not use this in anywhere else.
	 *
	 * @internal
	 */
	__internal__isPitching: boolean;
}

export interface LoaderResult {
	cacheable: boolean;
	content: string | Buffer;
	sourceMap?: string | SourceMap;
	additionalData?: AdditionalData;
	fileDependencies: string[];
	contextDependencies: string[];
	missingDependencies: string[];
	buildDependencies: string[];
}

export function createRawModuleRuleUses(
	uses: RuleSetUse,
	options: ComposeJsUseOptions
): RawModuleRuleUse[] {
	const normalizeRuleSetUseItem = (
		item: RuleSetUseItem
	): RuleSetLoaderWithOptions =>
		typeof item === "string" ? { loader: item } : item;
	const allUses = Array.isArray(uses)
		? [...uses].map(normalizeRuleSetUseItem)
		: [normalizeRuleSetUseItem(uses)];
	return createRawModuleRuleUsesImpl(allUses, options, allUses);
}

function createRawModuleRuleUsesImpl(
	uses: RuleSetLoaderWithOptions[],
	options: ComposeJsUseOptions,
	allUses: RuleSetLoaderWithOptions[]
): RawModuleRuleUse[] {
	if (!uses.length) {
		return [];
	}

	return uses.map(use => {
		if (
			typeof use.loader === "string" &&
			use.loader.startsWith(BUILTIN_LOADER_PREFIX)
		) {
			return createBuiltinUse(use);
		}
		return {
			jsLoader: {
				identifier: resolveStringifyLoaders(use, options.compiler)
			}
		};
	});
}

function resolveStringifyLoaders(
	use: RuleSetLoaderWithOptions,
	compiler: Compiler
) {
	const obj = parsePathQueryFragment(use.loader);
	obj.path = require.resolve(obj.path, {
		paths: [compiler.context]
	});
	let ident: string | null = null;

	if (use.options === null) obj.query = "";
	else if (use.options === undefined) obj.query = "";
	else if (typeof use.options === "string") obj.query = "?" + use.options;
	else if (use.ident) obj.query = "??" + (ident = use.ident);
	else if (typeof use.options === "object" && use.options.ident)
		obj.query = "??" + (ident = use.options.ident);
	else if (typeof use.options === "object")
		obj.query = "??" + (ident = generateRandomString(10));
	else obj.query = "?" + JSON.stringify(use.options);

	if (ident) {
		if (typeof use.options === "object") {
			compiler.ruleSet.references.set(ident, use.options);
		}
	}

	return obj.path + obj.query + obj.fragment;
}

function generateRandomString(length: number) {
	let result = "";
	const characters =
		"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
	const charactersLength = characters.length;
	for (let i = 0; i < length; i++) {
		result += characters.charAt(Math.floor(Math.random() * charactersLength));
	}
	return result;
}

function createBuiltinUse(use: RuleSetLoaderWithOptions): RawModuleRuleUse {
	assert(
		typeof use.loader === "string" &&
			use.loader.startsWith(BUILTIN_LOADER_PREFIX)
	);

	if (use.loader === `${BUILTIN_LOADER_PREFIX}sass-loader`) {
		(use.options ??= {} as any).__exePath = require.resolve(
			`sass-embedded-${process.platform}-${
				process.arch
			}/dart-sass-embedded/dart-sass-embedded${
				process.platform === "win32" ? ".bat" : ""
			}`
		);
	}

	return {
		builtinLoader: use.loader,
		options: JSON.stringify(use.options)
	};
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
