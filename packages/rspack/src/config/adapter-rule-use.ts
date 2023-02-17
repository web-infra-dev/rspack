import {
	JsAssetInfo,
	JsLoaderContext,
	JsLoaderResult,
	RawModuleRuleUse,
	RawOptions
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
import { concatErrorMsgAndStack, isPromiseLike } from "../util";
import { createHash } from "../util/createHash";
import Hash from "../util/hash";
import { absolutify, contextify, makePathsRelative } from "../util/identifier";
import { memoize } from "../util/memoize";
import { Mode, Resolve, RuleSetUse, RuleSetUseItem } from "./types";

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

export interface LoaderContext
	extends Pick<
		JsLoaderContext,
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
	cacheable(cacheable?: boolean): void;
	sourceMap: boolean;
	rootContext: string;
	context: string;
	loaderIndex: number;
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
	const allUses = [...uses].reverse();
	return createRawModuleRuleUsesImpl(allUses, options, allUses);
}

function createRawModuleRuleUsesImpl(
	uses: RuleSetUse,
	options: ComposeJsUseOptions,
	allUses: RuleSetUse
): RawModuleRuleUse[] {
	if (!uses.length) {
		return [];
	}
	const index = uses.findIndex(
		use =>
			typeof use.loader === "string" &&
			use.loader.startsWith(BUILTIN_LOADER_PREFIX)
	);
	if (index < 0) {
		// @ts-expect-error
		return [composeJsUse(uses, options, allUses)];
	}

	const before = uses.slice(0, index);
	const after = uses.slice(index + 1);
	return [
		composeJsUse(before, options, allUses),
		createBuiltinUse(uses[index]),
		...createRawModuleRuleUsesImpl(after, options, allUses)
	].filter((item): item is RawModuleRuleUse => Boolean(item));
}

function composeJsUse(
	uses: RuleSetUse,
	options: ComposeJsUseOptions,
	allUses: RuleSetUse
): RawModuleRuleUse | null {
	if (!uses.length) {
		return null;
	}

	async function loader(data: JsLoaderContext): Promise<JsLoaderResult> {
		const compiler = options.compiler;
		const resolver = compiler.resolverFactory.get("normal");
		const moduleContext = path.dirname(data.resourcePath);

		let cacheable: boolean = data.cacheable;
		let content: string | Buffer = data.content;
		let sourceMap: string | SourceMap | undefined =
			data.sourceMap?.toString("utf-8");
		let additionalData: AdditionalData | undefined = data.additionalData
			? JSON.parse(data.additionalData.toString("utf-8"))
			: undefined;

		// Loader is executed from right to left
		for (const use of uses) {
			const loaderIndex = allUses.indexOf(use);

			let loaderResult: LoaderResult;

			const p = new Promise<LoaderResult>((resolve, reject) => {
				let isDone = false;
				// Whether a `callback` or `async` is called
				let isSync = true;
				let isError = false; // internal error
				let reportedError = false;
				// @ts-expect-error
				const fileDependencies = [];
				// @ts-expect-error
				const contextDependencies = [];
				// @ts-expect-error
				const missingDependencies = [];
				// @ts-expect-error
				const buildDependencies = [];

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
						// @ts-expect-error
						fileDependencies,
						// @ts-expect-error
						contextDependencies,
						// @ts-expect-error
						missingDependencies,
						// @ts-expect-error
						buildDependencies
					});
				}

				const getResolveContext = () => {
					// FIXME: resolve's fileDependencies will includes lots of dir, '/', etc.
					return {
						fileDependencies: {
							// @ts-expect-error
							add: d => {
								// loaderContext.addDependency(d)
							}
						},
						contextDependencies: {
							// @ts-expect-error
							add: d => {
								// loaderContext.addContextDependency(d)
							}
						},
						missingDependencies: {
							// @ts-expect-error
							add: d => {
								// loaderContext.addMissingDependency(d)
							}
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
					// @ts-expect-error
					absolutify: (context, request) => {
						return context === moduleContext
							? getAbsolutifyInContext()(request)
							: getAbsolutify()(context, request);
					},
					// @ts-expect-error
					contextify: (context, request) => {
						return context === moduleContext
							? getContextifyInContext()(request)
							: getContextify()(context, request);
					},
					// @ts-expect-error
					createHash: type => {
						return createHash(
							// @ts-expect-error hashFunction should also avaiable in rust side, then we can make the type right
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
							: // deprecated usage so ignore the type
							  (use as any).query;
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
					// @ts-expect-error
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
					cacheable(value = true) {
						cacheable = value;
					},
					// @ts-expect-error
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
							error instanceof Error ? concatErrorMsgAndStack(error) : error;
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
								? concatErrorMsgAndStack(warning)
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

							if (this.sourceMap) {
								source = new SourceMapSource(
									content as any, // webpack-sources type declaration is wrong
									name,
									makePathsRelative(moduleContext, sourceMap, compiler) as any // webpack-sources type declaration is wrong
								);
							}
						} else {
							source = new RawSource(content as any); // webpack-sources type declaration is wrong
						}
						// @ts-expect-error
						compiler.compilation.emitAsset(name, source, assetInfo);
					},
					fs: compiler.inputFileSystem,
					utils,
					addBuildDependency(file) {
						buildDependencies.push(file);
					},
					addDependency(file) {
						fileDependencies.push(file);
					},
					addContextDependency(context) {
						contextDependencies.push(context);
					},
					addMissingDependency(missing) {
						missingDependencies.push(missing);
					},
					clearDependencies() {
						fileDependencies.length = 0;
						contextDependencies.length = 0;
						missingDependencies.length = 0;
					},
					getDependencies() {
						// @ts-expect-error
						return fileDependencies.slice();
					},
					getContextDependencies() {
						// @ts-expect-error
						return contextDependencies.slice();
					},
					getMissingDependencies() {
						// @ts-expect-error
						return missingDependencies.slice();
					},
					_compiler: compiler,
					_compilation: compiler.compilation
				};

				/**
				 * support loader as string
				 */
				let loader;
				if (typeof use.loader === "string") {
					try {
						const loaderPath = require.resolve(use.loader, {
							paths: [options.context]
						});
						const loaderModule = require(loaderPath);
						loader =
							typeof loaderModule === "function"
								? loaderModule
								: loaderModule.default;
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
								// @ts-expect-error
								buildDependencies,
								sourceMap,
								additionalData,
								cacheable,
								// @ts-expect-error
								fileDependencies,
								// @ts-expect-error
								contextDependencies,
								// @ts-expect-error
								missingDependencies
							});
							return;
						}
						if (isPromiseLike(result)) {
							return result.then(function (result) {
								resolve({
									content: result,
									// @ts-expect-error
									buildDependencies,
									sourceMap,
									additionalData,
									cacheable,
									// @ts-expect-error
									fileDependencies,
									// @ts-expect-error
									contextDependencies,
									// @ts-expect-error
									missingDependencies
								});
							}, reject);
						}
						return resolve({
							content: result,
							// @ts-expect-error
							buildDependencies,
							sourceMap,
							additionalData,
							cacheable,
							// @ts-expect-error
							fileDependencies,
							// @ts-expect-error
							contextDependencies,
							// @ts-expect-error
							missingDependencies
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
						// @ts-expect-error
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
				content = loaderResult.content ?? content;
				sourceMap = loaderResult.sourceMap ?? sourceMap;
				cacheable = loaderResult.cacheable ?? cacheable;

				data.fileDependencies.push(...loaderResult.fileDependencies);
				data.contextDependencies.push(...loaderResult.contextDependencies);
				data.missingDependencies.push(...loaderResult.missingDependencies);
				data.buildDependencies.push(...loaderResult.buildDependencies);
			}
		}

		return {
			cacheable: cacheable,
			fileDependencies: data.fileDependencies,
			contextDependencies: data.contextDependencies,
			missingDependencies: data.missingDependencies,
			buildDependencies: data.buildDependencies,
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

	return {
		jsLoader: {
			func: loader,
			name: uses.map(use => use.loader).join("!")
		}
	};
}

function createBuiltinUse(use: RuleSetUseItem): RawModuleRuleUse {
	assert(
		typeof use.loader === "string" &&
			use.loader.startsWith(BUILTIN_LOADER_PREFIX)
	);

	if (use.loader === `${BUILTIN_LOADER_PREFIX}sass-loader`) {
		(use.options ??= {} as any).__exePath = require.resolve(
			`@tmp-sass-embedded/${process.platform}-${
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

const toBuffer = (bufLike: string | Buffer): Buffer => {
	if (Buffer.isBuffer(bufLike)) {
		return bufLike;
	} else if (typeof bufLike === "string") {
		return Buffer.from(bufLike);
	}

	throw new Error("Buffer or string expected");
};

export function isUseSourceMap(devtool: RawOptions["devtool"]): boolean {
	return (
		devtool.includes("source-map") &&
		(devtool.includes("module") || !devtool.includes("cheap"))
	);
}

export function isUseSimpleSourceMap(devtool: RawOptions["devtool"]): boolean {
	return devtool.includes("source-map") && !isUseSourceMap(devtool);
}
