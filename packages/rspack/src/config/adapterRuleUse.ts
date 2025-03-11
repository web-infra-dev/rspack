import type { AssetInfo, RawModuleRuleUse, RawOptions } from "@rspack/binding";
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
import type { RspackOptionsNormalized } from "./normalization";
import type {
	Mode,
	PublicPath,
	Resolve,
	RuleSetLoaderWithOptions,
	RuleSetUseItem,
	Target
} from "./types";

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

export interface ImportModuleOptions {
	/**
	 * Specify a layer in which this module is placed/compiled
	 */
	layer?: string;
	/**
	 * The public path used for the built modules
	 */
	publicPath?: PublicPath;
	/**
	 * Target base uri
	 */
	baseUri?: string;
}

export interface LoaderContext<OptionsType = {}> {
	/**
	 * The version number of the loader API. Currently 2.
	 * This is useful for providing backwards compatibility. Using the version you can specify
	 * custom logic or fallbacks for breaking changes.
	 */
	version: 2;
	/**
	 * The path string of the current module.
	 * @example `'/abc/resource.js?query#hash'`.
	 */
	resource: string;
	/**
	 * The path string of the current module, excluding the query and fragment parameters.
	 * @example `'/abc/resource.js?query#hash'` in `'/abc/resource.js'`.
	 */
	resourcePath: string;
	/**
	 * The query parameter for the path string of the current module.
	 * @example `'?query'` in `'/abc/resource.js?query#hash'`.
	 */
	resourceQuery: string;
	/**
	 * The fragment parameter of the current module's path string.
	 * @example `'#hash'` in `'/abc/resource.js?query#hash'`.
	 */
	resourceFragment: string;
	/**
	 * Tells Rspack that this loader will be called asynchronously. Returns `this.callback`.
	 */
	async(): LoaderContextCallback;
	/**
	 * A function that can be called synchronously or asynchronously in order to return multiple
	 * results. The expected arguments are:
	 *
	 * 1. The first parameter must be `Error` or `null`, which marks the current module as a
	 * compilation failure.
	 * 2. The second argument is a `string` or `Buffer`, which indicates the contents of the file
	 * after the module has been processed by the loader.
	 * 3. The third parameter is a source map that can be processed by the loader.
	 * 4. The fourth parameter is ignored by Rspack and can be anything (e.g. some metadata).
	 */
	callback: LoaderContextCallback;
	/**
	 * A function that sets the cacheable flag.
	 * By default, the processing results of the loader are marked as cacheable.
	 * Calling this method and passing `false` turns off the loader's ability to
	 * cache processing results.
	 */
	cacheable(cacheable?: boolean): void;
	/**
	 * Tells if source map should be generated. Since generating source maps can be an expensive task,
	 * you should check if source maps are actually requested.
	 */
	sourceMap: boolean;
	/**
	 * The base path configured in Rspack config via `context`.
	 */
	rootContext: string;
	/**
	 * The directory path of the currently processed module, which changes with the
	 * location of each processed module.
	 * For example, if the loader is processing `/project/src/components/Button.js`,
	 * then the value of `this.context` would be `/project/src/components`.
	 */
	context: string | null;
	/**
	 * The index in the loaders array of the current loader.
	 */
	loaderIndex: number;
	remainingRequest: string;
	currentRequest: string;
	previousRequest: string;
	/**
	 * The module specifier string after being resolved.
	 * For example, if a `resource.js` is processed by `loader1.js` and `loader2.js`, the value of
	 * `this.request` will be `/path/to/loader1.js!/path/to/loader2.js!/path/to/resource.js`.
	 */
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
	/**
	 * The value of `mode` is read when Rspack is run.
	 * The possible values are: `'production'`, `'development'`, `'none'`
	 */
	mode?: Mode;
	/**
	 * The current compilation target. Passed from `target` configuration options.
	 */
	target?: Target;
	/**
	 * Whether HMR is enabled.
	 */
	hot?: boolean;
	/**
	 * Get the options passed in by the loader's user.
	 * @param schema To provide the best performance, Rspack does not perform the schema
	 * validation. If your loader requires schema validation, please call scheme-utils or
	 * zod on your own.
	 */
	getOptions(schema?: any): OptionsType;
	/**
	 * Resolve a module specifier.
	 * @param context The absolute path to a directory. This directory is used as the starting
	 * location for resolving.
	 * @param request The module specifier to be resolved.
	 * @param callback A callback function that gives the resolved path.
	 */
	resolve(
		context: string,
		request: string,
		callback: (
			arg0: null | Error,
			arg1?: string | false,
			arg2?: ResolveRequest
		) => void
	): void;
	/**
	 * Create a resolver like `this.resolve`.
	 */
	getResolve(
		options: Resolve
	):
		| ((context: string, request: string, callback: ResolveCallback) => void)
		| ((
				context: string,
				request: string
		  ) => Promise<string | false | undefined>);
	/**
	 * Get the logger of this compilation, through which messages can be logged.
	 */
	getLogger(name: string): Logger;
	/**
	 * Emit an error. Unlike `throw` and `this.callback(err)` in the loader, it does not
	 * mark the current module as a compilation failure, it just adds an error to Rspack's
	 * Compilation and displays it on the command line at the end of this compilation.
	 */
	emitError(error: Error): void;
	/**
	 * Emit a warning.
	 */
	emitWarning(warning: Error): void;
	/**
	 * Emit a new file. This method allows you to create new files during the loader execution.
	 */
	emitFile(
		name: string,
		content: string | Buffer,
		sourceMap?: string,
		assetInfo?: AssetInfo
	): void;
	/**
	 * Add a file as a dependency on the loader results so that any changes to them can be listened to.
	 * For example, `sass-loader`, `less-loader` use this trick to recompile when the imported style
	 * files change.
	 */
	addDependency(file: string): void;
	/**
	 * Alias of `this.addDependency()`.
	 */
	dependency(file: string): void;
	/**
	 * Add the directory as a dependency for the loader results so that any changes to the
	 * files in the directory can be listened to.
	 */
	addContextDependency(context: string): void;
	/**
	 * Add a currently non-existent file as a dependency of the loader result, so that its
	 * creation and any changes can be listened. For example, when a new file is created at
	 * that path, it will trigger a rebuild.
	 */
	addMissingDependency(missing: string): void;
	/**
	 * Removes all dependencies of the loader result.
	 */
	clearDependencies(): void;
	getDependencies(): string[];
	getContextDependencies(): string[];
	getMissingDependencies(): string[];
	addBuildDependency(file: string): void;
	/**
	 * Compile and execute a module at the build time.
	 * This is an alternative lightweight solution for the child compiler.
	 * `importModule` will return a Promise if no callback is provided.
	 *
	 * @example
	 * ```ts
	 * const modulePath = path.resolve(__dirname, 'some-module.ts');
	 * const moduleExports = await this.importModule(modulePath, {
	 *   // optional options
	 * });
	 * ```
	 */
	importModule<T = any>(
		request: string,
		options: ImportModuleOptions | undefined,
		callback: (err?: null | Error, exports?: T) => any
	): void;
	importModule<T = any>(
		request: string,
		options?: ImportModuleOptions
	): Promise<T>;
	/**
	 * Access to the `compilation` object's `inputFileSystem` property.
	 */
	fs: any;
	/**
	 * This is an experimental API and maybe subject to change.
	 * @experimental
	 */
	experiments: LoaderExperiments;
	/**
	 * Access to some utilities.
	 */
	utils: {
		/**
		 * Return a new request string using absolute paths when possible.
		 */
		absolutify: (context: string, request: string) => string;
		/**
		 * Return a new request string avoiding absolute paths when possible.
		 */
		contextify: (context: string, request: string) => string;
		/**
		 * Return a new Hash object from provided hash function.
		 */
		createHash: (algorithm?: string) => Hash;
	};
	/**
	 * The value depends on the loader configuration:
	 * - If the current loader was configured with an options object, `this.query` will
	 * point to that object.
	 * - If the current loader has no options, but was invoked with a query string, this
	 * will be a string starting with `?`.
	 */
	query: string | OptionsType;
	/**
	 * A data object shared between the pitch and the normal phase.
	 */
	data: unknown;
	/**
	 * Access to the current Compiler object of Rspack.
	 */
	_compiler: Compiler;
	/**
	 * Access to the current Compilation object of Rspack.
	 */
	_compilation: Compilation;
	/**
	 * @deprecated Hacky access to the Module object being loaded.
	 */
	_module: Module;
	/**
	 * Note: This is not a Rspack public API, maybe removed in future.
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
