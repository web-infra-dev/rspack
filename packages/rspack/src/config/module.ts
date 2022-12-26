import type {
	RawModuleRuleUse,
	RawModuleRule,
	RawModuleRuleCondition,
	RawModuleOptions,
	JsAssetInfo,
	JsLoaderContext,
	JsLoaderResult
} from "@rspack/binding";
import assert from "assert";
import { ResolveRequest } from "enhanced-resolve";
import path from "path";
import {
	OriginalSource,
	RawSource,
	Source,
	SourceMapSource
} from "webpack-sources";
import { Compiler } from "../compiler";
import { Logger } from "../logging/Logger";
import {
	ResolveOptions,
	Resolver,
	ResolverWithOptions
} from "../ResolverFactory";
import { isNil, isPromiseLike } from "../util";
import { createHash } from "../util/createHash";
import { createRawFromSource } from "../util/createSource";
import Hash from "../util/hash";
import { absolutify, contextify, makePathsRelative } from "../util/identifier";
import { memoize } from "../util/memoize";
import { ResolvedContext } from "./context";
import {
	isUseSimpleSourceMap,
	isUseSourceMap,
	ResolvedDevtool
} from "./devtool";
import { ResolvedMode } from "./mode";
import { Resolve, resolveResolveOptions } from "./resolve";
import { ResolvedTarget } from "./target";

export type Condition = string | RegExp;

export interface ModuleRule {
	name?: string;
	test?: Condition;
	include?: Condition | Condition[];
	exclude?: Condition | Condition[];
	resource?: Condition;
	resourceQuery?: Condition;
	use?: ModuleRuleUse[];
	type?: RawModuleRule["type"];
	resolve?: Resolve;
}

export interface Module {
	rules?: ModuleRule[];
	parser?: RawModuleOptions["parser"];
}

interface ResolvedModuleRule {
	test?: RawModuleRule["test"];
	include?: RawModuleRule["include"];
	exclude?: RawModuleRule["exclude"];
	resource?: RawModuleRule["resource"];
	resourceQuery?: RawModuleRule["resourceQuery"];
	use?: RawModuleRuleUse[];
	type?: RawModuleRule["type"];
}

export interface ResolvedModule {
	rules: ResolvedModuleRule[];
	parser?: RawModuleOptions["parser"];
}

interface LoaderContextInternal {
	// TODO: It's not a good way to do this, we should split the `source` into a separate type and avoid using `serde_json`, but it's a temporary solution.
	source: number[];
	sourceMap: string | undefined | null;
	additionalData: AdditionalData | undefined | null;
	resource: string;
	resourcePath: string;
	resourceQuery: string | null;
	resourceFragment: string | null;
	cacheable: boolean;
	buildDependencies: string[];
}
export interface LoaderContext
	extends Pick<
		LoaderContextInternal,
		"resource" | "resourcePath" | "resourceQuery" | "resourceFragment"
	> {
	version: 2;
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
	cacheable(cacheable: boolean): void;
	sourceMap: boolean;
	rootContext: string;
	context: string;
	loaderIndex: number;
	mode: ResolvedMode;
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
		options: ResolveOptions
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
}

const toBuffer = (bufLike: string | Buffer): Buffer => {
	if (Buffer.isBuffer(bufLike)) {
		return bufLike;
	} else if (typeof bufLike === "string") {
		return Buffer.from(bufLike);
	}

	throw new Error("Buffer or string expected");
};

export type GetCompiler = () => Compiler;

export interface ComposeJsUseOptions {
	devtool: ResolvedDevtool;
	context: ResolvedContext;
	target: ResolvedTarget;
	getCompiler: GetCompiler;
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
	// webpackAST: object;
}

export interface LoaderResult {
	cacheable: boolean;
	buildDependencies?: string[];
	content: string | Buffer;
	sourceMap?: string | SourceMap;
	additionalData?: AdditionalData;
	fileDependencies: string[];
	contextDependencies: string[];
	missingDependencies: string[];
}

function composeJsUse(
	uses: ModuleRuleUse[],
	options: ComposeJsUseOptions,
	allUses: ModuleRuleUse[]
): RawModuleRuleUse | null {
	if (!uses.length) {
		return null;
	}

	async function loader(data: JsLoaderContext): Promise<JsLoaderResult> {
		const compiler = options.getCompiler();
		const resolver = compiler.resolverFactory.get("normal");
		const moduleContext = path.dirname(data.resourcePath);

		let cacheable: boolean = data.cacheable;
		let buildDependencies = data.buildDependencies;
		let content: string | Buffer = data.content;
		let sourceMap: string | SourceMap | undefined =
			data.sourceMap?.toString("utf-8");
		let additionalData: AdditionalData | undefined = data.additionalData
			? JSON.parse(data.additionalData.toString("utf-8"))
			: undefined;

		// Loader is executed from right to left
		for (const use of uses) {
			assert("loader" in use);
			const loaderIndex = allUses.indexOf(use);

			let loaderResult: LoaderResult;

			const p = new Promise<LoaderResult>((resolve, reject) => {
				let isDone = false;
				// Whether a `callback` or `async` is called
				let isSync = true;
				let isError = false; // internal error
				let reportedError = false;

				const fileDependencies = [];
				const contextDependencies = [];
				const missingDependencies = [];

				function callback(
					err: Error | null,
					content: string | Buffer,
					sourceMap?: string | SourceMap,
					additionalData?: AdditionalData
				) {
					if (isDone) {
						if (reportedError) return; // ignore
						err = new Error("callback(): The callback was already called.");
					}
					isSync = false;
					isDone = true;

					if (err) {
						isError = true;
						reject(err);
						return;
					}

					resolve({
						cacheable,
						content,
						sourceMap,
						additionalData,
						fileDependencies,
						contextDependencies,
						missingDependencies,
					});
				}

				const getResolveContext = () => {
					return {
						fileDependencies: {
							add: d => loaderContext.addDependency(d)
						},
						contextDependencies: {
							add: d => loaderContext.addContextDependency(d)
						},
						missingDependencies: {
							add: d => loaderContext.addMissingDependency(d)
						}
					};
				};

				const getAbsolutify = memoize(() =>
					absolutify.bindCache(compiler.root)
				);
				const getAbsolutifyInContext = memoize(() =>
					absolutify.bindContextCache(moduleContext, compiler.root)
				);
				const getContextify = memoize(() =>
					contextify.bindCache(compiler.root)
				);
				const getContextifyInContext = memoize(() =>
					contextify.bindContextCache(moduleContext, compiler.root)
				);
				const utils = {
					absolutify: (context, request) => {
						return context === moduleContext
							? getAbsolutifyInContext()(request)
							: getAbsolutify()(context, request);
					},
					contextify: (context, request) => {
						return context === moduleContext
							? getContextifyInContext()(request)
							: getContextify()(context, request);
					},
					createHash: type => {
						return createHash(
							type || compiler.compilation.outputOptions.hashFunction
						);
					}
				};

				const loaderContext: LoaderContext = {
					version: 2,
					sourceMap: isUseSourceMap(options.devtool),
					resourcePath: data.resourcePath,
					resource: data.resource,
					// Return an empty string if there is no query or fragment
					resourceQuery: data.resourceQuery || "",
					resourceFragment: data.resourceFragment || "",
					loaderIndex,
					mode: compiler.options.mode,
					hot: compiler.options.devServer?.hot,
					getOptions(schema) {
						let { options } = use;

						if (options === null || options === undefined) {
							options = {};
						}

						if (schema) {
							let name = "Loader";
							let baseDataPath = "options";
							let match;
							if (schema.title && (match = /^(.+) (.+)$/.exec(schema.title))) {
								[, name, baseDataPath] = match;
							}
							const { validate } = require("schema-utils");
							validate(schema, options, {
								name,
								baseDataPath
							});
						}

						return options;
					},
					get query() {
						return use.options && typeof use.options === "object"
							? use.options
							: use.query;
					},
					get data() {
						return use.data;
					},
					resolve(context, request, callback) {
						resolver.resolve(
							{},
							context,
							request,
							getResolveContext(),
							callback
						);
					},
					getResolve(options) {
						const child = options ? resolver.withOptions(options) : resolver;
						return (context, request, callback) => {
							if (callback) {
								child.resolve(
									{},
									context,
									request,
									getResolveContext(),
									callback
								);
							} else {
								return new Promise((resolve, reject) => {
									child.resolve(
										{},
										context,
										request,
										getResolveContext(),
										(err, result) => {
											if (err) reject(err);
											else resolve(result);
										}
									);
								});
							}
						};
					},
					getLogger(name) {
						return compiler.getInfrastructureLogger(() =>
							[name, data.resource].filter(Boolean).join("|")
						);
					},
					cacheable(value) {
						cacheable = value;
					},
					async() {
						if (isDone) {
							if (reportedError) return; // ignore
							reject(new Error("async(): The callback was already called."));
						}
						isSync = false;
						return callback;
					},
					callback,
					rootContext: options.context,
					context: moduleContext,
					emitError(error) {
						const title = "Module Error";
						const message =
							error instanceof Error
								? `${error.message}${error.stack ? `\n${error.stack}` : ""}`
								: error;
						compiler.compilation.pushDiagnostic(
							"error",
							title,
							`${message}\n(from: ${use.loader})`
						);
					},
					emitWarning(warning) {
						const title = "Module Warning";
						const message =
							warning instanceof Error
								? `${warning.message}${
										warning.stack ? `\n${warning.stack}` : ""
								  }`
								: warning;
						compiler.compilation.pushDiagnostic(
							"warning",
							title,
							`${message}\n(from: ${use.loader})`
						);
					},
					emitFile(name, content, sourceMap?, assetInfo?) {
						let source: Source;
						if (sourceMap) {
							if (
								typeof sourceMap === "string" &&
								(loaderContext.sourceMap ||
									isUseSimpleSourceMap(options.devtool))
							) {
								source = new OriginalSource(
									content,
									makePathsRelative(moduleContext, sourceMap, compiler)
								);
							}

							if (this.useSourceMap) {
								source = new SourceMapSource(
									content as any, // webpack-sources type declaration is wrong
									name,
									makePathsRelative(moduleContext, sourceMap, compiler) as any // webpack-sources type declaration is wrong
								);
							}
						} else {
							source = new RawSource(content as any); // webpack-sources type declaration is wrong
						}

						compiler.compilation.emitAsset(name, source, assetInfo);
					},
					fs: compiler.inputFileSystem,
					utils,
					addBuildDependency(file) {
						buildDependencies.push(file);
					},
					addDependency(file) {
						fileDependencies.push(file)
					},
					addContextDependency(context) {
						contextDependencies.push(context)
					},
					addMissingDependency(missing) {
						missingDependencies.push(missing)
					},
					clearDependencies() {
						fileDependencies.length = 0;
						contextDependencies.length = 0;
						missingDependencies.length = 0;
					},
					getDependencies() {
						return fileDependencies.slice();
					},
					getContextDependencies() {
						return contextDependencies.slice();
					},
					getMissingDependencies() {
						return missingDependencies.slice();
					},
				};

				/**
				 * support loader as string
				 */
				let loader: Loader | undefined;
				if (typeof use.loader === "string") {
					try {
						let loaderPath = require.resolve(use.loader, {
							paths: [options.context]
						});
						loader = require(loaderPath);
					} catch (err) {
						reject(err);
						return;
					}
				} else {
					loader = use.loader;
				}

				let result: Promise<string | Buffer> | string | Buffer | undefined =
					undefined;
				try {
					result = loader.apply(loaderContext, [
						loader.raw ? Buffer.from(content) : content.toString("utf-8"),
						sourceMap,
						additionalData
					]);
					if (isSync) {
						isDone = true;
						if (result === undefined) {
							resolve({
								content,
								buildDependencies,
								sourceMap,
								additionalData,
								cacheable,
								fileDependencies,
								contextDependencies,
								missingDependencies,
							});
							return;
						}
						if (isPromiseLike(result)) {
							return result.then(function (result) {
								resolve({
									content: result,
									buildDependencies,
									sourceMap,
									additionalData,
									cacheable,
									fileDependencies,
									contextDependencies,
									missingDependencies,
								});
							}, reject);
						}
						return resolve({
							content: result,
							buildDependencies,
							sourceMap,
							additionalData,
							cacheable,
							fileDependencies,
							contextDependencies,
							missingDependencies,
						});
					}
				} catch (err) {
					if (isError) {
						reject(err);
						return;
					}
					if (isDone) {
						// loader is already "done", so we cannot use the callback function
						// for better debugging we print the error on the console
						if (typeof err === "object" && err.stack) console.error(err.stack);
						else console.error(err);
						reject(err);
						return;
					}
					isDone = true;
					reportedError = true;
					reject(err);
				}
			});

			if ((loaderResult = await p)) {
				additionalData =
					(typeof loaderResult.additionalData === "string"
						? JSON.parse(loaderResult.additionalData)
						: loaderResult.additionalData) ?? additionalData;
				buildDependencies = loaderResult.buildDependencies ?? buildDependencies;
				content = loaderResult.content ?? content;
				sourceMap = loaderResult.sourceMap ?? sourceMap;
				cacheable = loaderResult.cacheable ?? cacheable;

				compiler.watcher.add([
					...loaderResult.fileDependencies,
					...loaderResult.contextDependencies,
					...loaderResult.missingDependencies,
				]);
			}
		}

		return {
			cacheable: cacheable,
			buildDependencies: buildDependencies,
			content: toBuffer(content),
			sourceMap: sourceMap
				? toBuffer(
						typeof sourceMap === "string"
							? sourceMap
							: JSON.stringify(sourceMap)
				  )
				: undefined,
			additionalData: additionalData
				? toBuffer(JSON.stringify(additionalData))
				: undefined
		};
	}

	loader.displayName = `NodeLoaderAdapter(${uses
		.map(item => {
			assert("loader" in item);
			let loader: Loader | null;
			if (typeof item.loader === "string") {
				try {
					const path = require.resolve(item.loader, {
						paths: [options.context]
					});
					loader = require(path);
				} catch (e) {
					loader = null;
				}
			} else {
				loader = item.loader;
			}
			return loader?.displayName || loader?.name || "unknown-loader";
		})
		.join(" -> ")})`;
	return {
		loader
	};
}

export interface Loader {
	(
		this: LoaderContext,
		content: string | Buffer,
		sourceMap?: string | SourceMap,
		additionalData?: AdditionalData
	): void;
	displayName?: string;
	raw?: boolean;
}

type BuiltinLoader = string;

type ModuleRuleUse =
	| {
			builtinLoader: BuiltinLoader;
			options?: unknown;
			name?: string;
	  }
	| {
			// String represents a path to the loader
			loader: Loader | string;
			options?: unknown;
			name?: string;
			query?: string;
			data?: unknown;
	  };

export function createRawModuleRuleUses(
	uses: ModuleRuleUse[],
	options: ComposeJsUseOptions
): RawModuleRuleUse[] {
	const allUses = [...uses].reverse();
	return createRawModuleRuleUsesImpl(allUses, options, allUses);
}

function createRawModuleRuleUsesImpl(
	uses: ModuleRuleUse[],
	options: ComposeJsUseOptions,
	allUses: ModuleRuleUse[]
): RawModuleRuleUse[] {
	if (!uses.length) {
		return [];
	}
	const index = uses.findIndex(use => "builtinLoader" in use);
	if (index < 0) {
		return [composeJsUse(uses, options, allUses)];
	}

	const before = uses.slice(0, index);
	const after = uses.slice(index + 1);
	return [
		composeJsUse(before, options, allUses),
		createNativeUse(uses[index]),
		...createRawModuleRuleUsesImpl(after, options, allUses)
	].filter((item): item is RawModuleRuleUse => Boolean(item));
}

function createNativeUse(use: ModuleRuleUse): RawModuleRuleUse {
	assert("builtinLoader" in use);

	if (use.builtinLoader === "sass-loader") {
		(use.options ??= {} as any).__exePath = require.resolve(
			`@tmp-sass-embedded/${process.platform}-${
				process.arch
			}/dart-sass-embedded/dart-sass-embedded${
				process.platform === "win32" ? ".bat" : ""
			}`
		);
	}

	return {
		builtinLoader: use.builtinLoader,
		options: JSON.stringify(use.options)
	};
}

function resolveModuleRuleCondition(
	condition: Condition
): RawModuleRuleCondition {
	if (typeof condition === "string") {
		return {
			type: "string",
			matcher: condition
		};
	}

	if (condition instanceof RegExp) {
		return {
			type: "regexp",
			matcher: condition.source
		};
	}

	throw new Error(
		`Unsupported condition type ${typeof condition}, value: ${condition}`
	);
}

function resolveModuleRuleConditions(
	conditions: Condition[]
): RawModuleRuleCondition[] {
	return conditions.map(resolveModuleRuleCondition);
}

export function resolveModuleOptions(
	module: Module = {},
	options: ComposeJsUseOptions
): ResolvedModule {
	const rules = (module.rules ?? []).map(rule => {
		// FIXME: use error handler instead of throwing
		if ((rule as any)?.loader) {
			throw new Error("`Rule.loader` is not supported, use `Rule.use` instead");
		}

		if ((rule as any)?.uses) {
			throw new Error(
				"`Rule.uses` is deprecated for aligning with webpack, use `Rule.use` instead"
			);
		}

		return {
			...rule,
			test: isNil(rule.test) ? null : resolveModuleRuleCondition(rule.test),
			include: isNil(rule.include)
				? null
				: Array.isArray(rule.include)
				? resolveModuleRuleConditions(rule.include)
				: [resolveModuleRuleCondition(rule.include)],
			exclude: isNil(rule.exclude)
				? null
				: Array.isArray(rule.exclude)
				? resolveModuleRuleConditions(rule.exclude)
				: [resolveModuleRuleCondition(rule.exclude)],
			resource: isNil(rule.resource)
				? null
				: resolveModuleRuleCondition(rule.resource),
			resourceQuery: isNil(rule.resourceQuery)
				? null
				: resolveModuleRuleCondition(rule.resourceQuery),
			use: createRawModuleRuleUses(rule.use || [], options),
			resolve: isNil(rule.resolve)
				? null
				: resolveResolveOptions(rule.resolve, options)
		};
	});
	return {
		parser: module.parser,
		rules
	};
}
