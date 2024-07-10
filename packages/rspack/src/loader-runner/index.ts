/**
 * The following code is modified based on
 * https://github.com/webpack/loader-runner
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/loader-runner/blob/main/LICENSE
 */

import querystring from "node:querystring";

import assert from "assert";
import { promisify } from "util";
import {
	type JsLoaderContext,
	type JsLoaderItem,
	JsLoaderState,
	JsRspackSeverity
} from "@rspack/binding";
import {
	OriginalSource,
	RawSource,
	type Source,
	SourceMapSource
} from "webpack-sources";

import type { Compilation } from "../Compilation";
import type { Compiler } from "../Compiler";
import { Module } from "../Module";
import { NormalModule } from "../NormalModule";
import {
	JsDiagnostic,
	NonErrorEmittedError,
	type RspackError
} from "../RspackError";
import {
	BUILTIN_LOADER_PREFIX,
	type LoaderContext,
	isUseSimpleSourceMap,
	isUseSourceMap
} from "../config/adapterRuleUse";
import {
	concatErrorMsgAndStack,
	isNil,
	serializeObject,
	stringifyLoaderObject,
	toBuffer,
	toObject
} from "../util";
import { createHash } from "../util/createHash";
import {
	absolutify,
	contextify,
	makePathsRelative,
	parseResourceWithoutFragment
} from "../util/identifier";
import { memoize } from "../util/memoize";
import loadLoader from "./loadLoader";

function createLoaderObject(
	loader: JsLoaderItem,
	compiler: Compiler
): LoaderObject {
	var obj: any = {
		path: null,
		query: null,
		fragment: null,
		options: null,
		ident: null,
		normal: null,
		pitch: null,
		raw: null,
		data: null,
		pitchExecuted: false,
		normalExecuted: false
	};
	Object.defineProperty(obj, "request", {
		enumerable: true,
		get: () =>
			obj.path.replace(/#/g, "\u200b#") +
			obj.query.replace(/#/g, "\u200b#") +
			obj.fragment,
		set: value => {
			var splittedRequest = parseResourceWithoutFragment(value.request);
			obj.path = splittedRequest.path;
			obj.query = splittedRequest.query;
			obj.fragment = splittedRequest.fragment || "";
			obj.options =
				obj.options === null
					? splittedRequest.query
						? splittedRequest.query.slice(1)
						: undefined
					: obj.options;

			if (typeof obj.options === "string" && obj.options[0] === "?") {
				const ident = obj.options.slice(1);
				if (ident === "[[missing ident]]") {
					throw new Error(
						"No ident is provided by referenced loader. " +
							"When using a function for Rule.use in config you need to " +
							"provide an 'ident' property for referenced loader options."
					);
				}
				obj.options = compiler.__internal__ruleSet.references.get(ident);
				if (obj.options === undefined) {
					throw new Error("Invalid ident is provided by referenced loader");
				}
				obj.ident = ident;
			}

			obj.type = value.type;
			if (obj.options === null) obj.query = "";
			else if (obj.options === undefined) obj.query = "";
			else if (typeof obj.options === "string") obj.query = "?" + obj.options;
			else if (obj.ident) obj.query = "??" + obj.ident;
			else if (typeof obj.options === "object" && obj.options.ident)
				obj.query = "??" + obj.options.ident;
			else obj.query = "?" + JSON.stringify(obj.options);
		}
	});
	obj.request = loader;
	if (Object.preventExtensions) {
		Object.preventExtensions(obj);
	}
	return obj;
}

export class LoaderObject {
	request: string;
	path: string;
	query: string;
	fragment: string;
	options?: string | object;
	ident: string;
	normal?: Function;
	pitch?: Function;
	raw?: boolean;
	type?: "module" | "commonjs";
	#loaderItem: JsLoaderItem;

	constructor(loaderItem: JsLoaderItem, compiler: Compiler) {
		const {
			request,
			path,
			query,
			fragment,
			options,
			ident,
			normal,
			pitch,
			raw,
			type
		} = createLoaderObject(loaderItem, compiler);
		this.request = request;
		this.path = path;
		this.query = query;
		this.fragment = fragment;
		this.options = options;
		this.ident = ident;
		this.normal = normal;
		this.pitch = pitch;
		this.raw = raw;
		this.type = type;
		this.#loaderItem = loaderItem;
	}

	get pitchExecuted() {
		return this.#loaderItem.pitchExecuted;
	}

	set pitchExecuted(value: boolean) {
		assert(value);
		this.#loaderItem.pitchExecuted = true;
	}

	get normalExecuted() {
		return this.#loaderItem.normalExecuted;
	}

	set normalExecuted(value: boolean) {
		assert(value);
		this.#loaderItem.normalExecuted = true;
	}

	// A data object shared between the pitch and the normal phase
	get data() {
		return new Proxy((this.#loaderItem.data = this.#loaderItem.data ?? {}), {
			set: (_, property, value) => {
				if (typeof property === "string") {
					this.#loaderItem.data[property] = value;
				}
				return true;
			},
			get: (_, property) => {
				if (typeof property === "string") {
					return this.#loaderItem.data[property];
				}
			}
		});
	}

	// A data object shared between the pitch and the normal phase
	set data(data: any) {
		this.#loaderItem.data = data;
	}

	shouldYield() {
		return this.request.startsWith(BUILTIN_LOADER_PREFIX);
	}

	static __from_binding(
		loaderItem: JsLoaderItem,
		compiler: Compiler
	): LoaderObject {
		return new this(loaderItem, compiler);
	}

	static __to_binding(loader: LoaderObject): JsLoaderItem {
		return loader.#loaderItem;
	}
}

class JsSourceMap {
	static __from_binding(map?: Buffer) {
		return isNil(map) ? undefined : toObject(map);
	}

	static __to_binding(map?: object) {
		return serializeObject(map);
	}
}

const loadLoaderAsync: (loaderObject: LoaderObject) => Promise<void> =
	promisify(loadLoader);

const runSyncOrAsync = promisify(function runSyncOrAsync(
	fn: Function,
	context: LoaderContext,
	args: any[],
	callback: (err: Error | null, args: any[]) => void
) {
	let isSync = true;
	let isDone = false;
	let isError = false; // internal error
	let reportedError = false;
	// @ts-expect-error loader-runner leverages `arguments` to achieve the same functionality.
	context.async = function async() {
		if (isDone) {
			if (reportedError) return; // ignore
			throw new Error("async(): The callback was already called.");
		}
		isSync = false;
		return innerCallback;
	};
	const innerCallback = (context.callback = (err, ...args) => {
		if (isDone) {
			if (reportedError) return; // ignore
			throw new Error("callback(): The callback was already called.");
		}
		isDone = true;
		isSync = false;
		try {
			// @ts-expect-error
			callback(err, args);
		} catch (e) {
			isError = true;
			throw e;
		}
	});
	try {
		const result = (function LOADER_EXECUTION() {
			return fn.apply(context, args);
		})();
		if (isSync) {
			isDone = true;
			if (result === undefined) {
				// @ts-expect-error
				callback();
				return;
			}
			if (
				result &&
				typeof result === "object" &&
				typeof result.then === "function"
			) {
				result.then((r: unknown) => {
					callback(null, [r]);
				}, callback);
				return;
			}
			callback(null, [result]);
			return;
		}
	} catch (e: unknown) {
		if (isError) throw e;
		if (isDone) {
			// loader is already "done", so we cannot use the callback function
			// for better debugging we print the error on the console
			if (e instanceof Error) console.error(e.stack);
			else console.error(e);
			return;
		}
		isDone = true;
		reportedError = true;
		// @ts-expect-error
		callback(e);
	}
});

function dirname(path: string) {
	if (path === "/") return "/";
	const i = path.lastIndexOf("/");
	const j = path.lastIndexOf("\\");
	const i2 = path.indexOf("/");
	const j2 = path.indexOf("\\");
	const idx = i > j ? i : j;
	const idx2 = i > j ? i2 : j2;
	if (idx < 0) return path;
	if (idx === idx2) return path.slice(0, idx + 1);
	return path.slice(0, idx);
}

function getCurrentLoader(
	loaderContext: LoaderContext,
	index = loaderContext.loaderIndex
) {
	if (
		loaderContext.loaders &&
		loaderContext.loaders.length &&
		index < loaderContext.loaders.length &&
		index >= 0 &&
		loaderContext.loaders[index]
	) {
		return loaderContext.loaders[index];
	}
	return null;
}

export async function runLoaders(
	compiler: Compiler,
	context: JsLoaderContext
): Promise<JsLoaderContext> {
	const loaderState = context.loaderState;

	//
	const { resource } = context.resourceData;
	const splittedResource = resource && parsePathQueryFragment(resource);
	const resourcePath = splittedResource ? splittedResource.path : undefined;
	const resourceQuery = splittedResource ? splittedResource.query : undefined;
	const resourceFragment = splittedResource
		? splittedResource.fragment
		: undefined;
	const contextDirectory = resourcePath ? dirname(resourcePath) : null;

	// execution state
	const fileDependencies = context.fileDependencies;
	const contextDependencies = context.contextDependencies;
	const missingDependencies = context.missingDependencies;
	const buildDependencies = context.buildDependencies;
	const assetFilenames = context.assetFilenames;

	/// Construct `loaderContext`
	const loaderContext = {} as LoaderContext;

	loaderContext.loaders = context.loaderItems.map(item => {
		return LoaderObject.__from_binding(item, compiler);
	});

	loaderContext.hot = context.hot;
	loaderContext.context = contextDirectory;
	loaderContext.resourcePath = resourcePath!;
	loaderContext.resourceQuery = resourceQuery!;
	loaderContext.resourceFragment = resourceFragment!;
	loaderContext.dependency = loaderContext.addDependency =
		function addDependency(file) {
			fileDependencies.push(file);
		};
	loaderContext.addContextDependency = function addContextDependency(context) {
		contextDependencies.push(context);
	};
	loaderContext.addMissingDependency = function addMissingDependency(context) {
		missingDependencies.push(context);
	};
	loaderContext.addBuildDependency = function addBuildDependency(file) {
		buildDependencies.push(file);
	};
	loaderContext.getDependencies = function getDependencies() {
		return fileDependencies.slice();
	};
	loaderContext.getContextDependencies = function getContextDependencies() {
		return contextDependencies.slice();
	};
	loaderContext.getMissingDependencies = function getMissingDependencies() {
		return missingDependencies.slice();
	};
	loaderContext.clearDependencies = function clearDependencies() {
		fileDependencies.length = 0;
		contextDependencies.length = 0;
		missingDependencies.length = 0;
		context.cacheable = true;
	};
	loaderContext.importModule = function importModule(
		request,
		options,
		callback
	) {
		if (!callback) {
			return new Promise((resolve, reject) => {
				compiler
					._lastCompilation!.__internal_getInner()
					.importModule(
						request,
						options.publicPath,
						options.baseUri,
						context._module.moduleIdentifier,
						loaderContext.context,
						(err, res) => {
							if (err) reject(err);
							else {
								for (const dep of res.buildDependencies) {
									this.addBuildDependency(dep);
								}
								for (const dep of res.contextDependencies) {
									this.addContextDependency(dep);
								}
								for (const dep of res.missingDependencies) {
									this.addMissingDependency(dep);
								}
								for (const dep of res.fileDependencies) {
									this.addDependency(dep);
								}
								if (res.cacheable === false) {
									this.cacheable(false);
								}
								assetFilenames.push(...res.assets);

								resolve(compiler.__internal__getModuleExecutionResult(res.id));
							}
						}
					);
			});
		}
		return compiler
			._lastCompilation!.__internal_getInner()
			.importModule(
				request,
				options.publicPath,
				options.baseUri,
				context._module.moduleIdentifier,
				loaderContext.context,
				(err, res) => {
					if (err) {
						callback(err, undefined);
					} else {
						for (const dep of res.buildDependencies) {
							this.addBuildDependency(dep);
						}
						for (const dep of res.contextDependencies) {
							this.addContextDependency(dep);
						}
						for (const dep of res.missingDependencies) {
							this.addMissingDependency(dep);
						}
						for (const dep of res.fileDependencies) {
							this.addDependency(dep);
						}
						if (res.cacheable === false) {
							this.cacheable(false);
						}
						assetFilenames.push(...res.assets);

						callback(
							undefined,
							compiler.__internal__getModuleExecutionResult(res.id)
						);
					}
				}
			);
	};
	Object.defineProperty(loaderContext, "resource", {
		enumerable: true,
		get: () => {
			if (loaderContext.resourcePath === undefined) return undefined;
			return (
				loaderContext.resourcePath.replace(/#/g, "\u200b#") +
				loaderContext.resourceQuery.replace(/#/g, "\u200b#") +
				loaderContext.resourceFragment
			);
		},
		set: value => {
			const splittedResource = value && parsePathQueryFragment(value);
			loaderContext.resourcePath = splittedResource
				? splittedResource.path
				: undefined;
			loaderContext.resourceQuery = splittedResource
				? splittedResource.query
				: undefined;
			loaderContext.resourceFragment = splittedResource
				? splittedResource.fragment
				: undefined;
		}
	});
	Object.defineProperty(loaderContext, "request", {
		enumerable: true,
		get: () =>
			loaderContext.loaders
				.map(o => o.request)
				.concat(loaderContext.resource || "")
				.join("!")
	});
	Object.defineProperty(loaderContext, "remainingRequest", {
		enumerable: true,
		get: () => {
			if (
				loaderContext.loaderIndex >= loaderContext.loaders.length - 1 &&
				!loaderContext.resource
			)
				return "";
			return loaderContext.loaders
				.slice(loaderContext.loaderIndex + 1)
				.map(o => o.request)
				.concat(loaderContext.resource || "")
				.join("!");
		}
	});
	Object.defineProperty(loaderContext, "currentRequest", {
		enumerable: true,
		get: () =>
			loaderContext.loaders
				.slice(loaderContext.loaderIndex)
				.map(o => o.request)
				.concat(loaderContext.resource || "")
				.join("!")
	});
	Object.defineProperty(loaderContext, "previousRequest", {
		enumerable: true,
		get: () =>
			loaderContext.loaders
				.slice(0, loaderContext.loaderIndex)
				.map(o => o.request)
				.join("!")
	});
	Object.defineProperty(loaderContext, "query", {
		enumerable: true,
		get: () => {
			const entry = loaderContext.loaders[loaderContext.loaderIndex];
			return entry.options && typeof entry.options === "object"
				? entry.options
				: entry.query;
		}
	});
	loaderContext.version = 2;
	loaderContext.sourceMap = compiler.options.devtool
		? isUseSourceMap(compiler.options.devtool)
		: false;
	loaderContext.mode = compiler.options.mode;
	Object.assign(loaderContext, compiler.options.loader);

	const getResolveContext = () => {
		return {
			fileDependencies: {
				// @ts-expect-error: Mocking insert-only `Set<T>`
				add: d => {
					loaderContext.addDependency(d);
				}
			},
			contextDependencies: {
				// @ts-expect-error: Mocking insert-only `Set<T>`
				add: d => {
					loaderContext.addContextDependency(d);
				}
			},
			missingDependencies: {
				// @ts-expect-error: Mocking insert-only `Set<T>`
				add: d => {
					loaderContext.addMissingDependency(d);
				}
			}
		};
	};

	const resolver = compiler._lastCompilation!.resolverFactory.get("normal");
	loaderContext.resolve = function resolve(context, request, callback) {
		resolver.resolve({}, context, request, getResolveContext(), callback);
	};
	// @ts-expect-error TODO
	loaderContext.getResolve = function getResolve(options) {
		const child = options ? resolver.withOptions(options as any) : resolver;
		return (context, request, callback) => {
			if (callback) {
				child.resolve({}, context, request, getResolveContext(), callback);
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
	};
	loaderContext.getLogger = function getLogger(name) {
		return compiler._lastCompilation!.getLogger(
			[name, resource].filter(Boolean).join("|")
		);
	};
	loaderContext.rootContext = compiler.context;
	loaderContext.emitError = function emitError(error) {
		if (!(error instanceof Error)) {
			error = new NonErrorEmittedError(error);
		}
		let hasStack = !!error.stack;
		error.name = "ModuleError";
		error.message = `${error.message} (from: ${stringifyLoaderObject(
			loaderContext.loaders[loaderContext.loaderIndex]
		)})`;
		hasStack && Error.captureStackTrace(error);
		error = concatErrorMsgAndStack(error);
		(error as RspackError).moduleIdentifier = this._module.identifier();
		compiler._lastCompilation!.__internal__pushDiagnostic({
			error,
			severity: JsRspackSeverity.Error
		});
	};
	loaderContext.emitWarning = function emitWarning(warning) {
		if (!(warning instanceof Error)) {
			warning = new NonErrorEmittedError(warning);
		}
		let hasStack = !!warning.stack;
		warning.name = "ModuleWarning";
		warning.message = `${warning.message} (from: ${stringifyLoaderObject(
			loaderContext.loaders[loaderContext.loaderIndex]
		)})`;
		hasStack && Error.captureStackTrace(warning);
		warning = concatErrorMsgAndStack(warning);
		(warning as RspackError).moduleIdentifier = this._module.identifier();
		compiler._lastCompilation!.__internal__pushDiagnostic({
			error: warning,
			severity: JsRspackSeverity.Warn
		});
	};
	loaderContext.emitFile = function emitFile(
		name,
		content,
		sourceMap?,
		assetInfo?
	) {
		let source: Source;
		if (sourceMap) {
			if (
				typeof sourceMap === "string" &&
				(loaderContext.sourceMap ||
					(compiler.options.devtool &&
						isUseSimpleSourceMap(compiler.options.devtool)))
			) {
				source = new OriginalSource(
					content,
					makePathsRelative(contextDirectory!, sourceMap, compiler)
				);
			}

			if (this.sourceMap) {
				source = new SourceMapSource(
					// @ts-expect-error webpack-sources type declaration is wrong
					content,
					name,
					makePathsRelative(contextDirectory!, sourceMap, compiler)
				);
			}
		} else {
			source = new RawSource(
				// @ts-expect-error webpack-sources type declaration is wrong
				content
			);
		}
		assetFilenames.push(name),
			// @ts-expect-error
			compiler._lastCompilation.emitAsset(name, source, assetInfo);
	};
	loaderContext.fs = compiler.inputFileSystem;

	const getAbsolutify = memoize(() => absolutify.bindCache(compiler.root));
	const getAbsolutifyInContext = memoize(() =>
		absolutify.bindContextCache(contextDirectory!, compiler.root)
	);
	const getContextify = memoize(() => contextify.bindCache(compiler.root));
	const getContextifyInContext = memoize(() =>
		contextify.bindContextCache(contextDirectory!, compiler.root)
	);

	loaderContext.utils = {
		absolutify: (context, request) => {
			return context === contextDirectory
				? getAbsolutifyInContext()(request)
				: getAbsolutify()(context, request);
		},
		contextify: (context, request) => {
			return context === contextDirectory
				? getContextifyInContext()(request)
				: getContextify()(context, request);
		},
		createHash: type => {
			return createHash(
				type || compiler._lastCompilation!.outputOptions.hashFunction
			);
		}
	};

	loaderContext._compiler = compiler;
	loaderContext._compilation = compiler._lastCompilation!;
	loaderContext._module = Module.__from_binding(
		context._module,
		compiler._lastCompilation
	);

	loaderContext.getOptions = () => {
		const loader = getCurrentLoader(loaderContext);
		let options = loader?.options;

		if (typeof options === "string") {
			if (options.startsWith("{") && options.endsWith("}")) {
				try {
					const parseJson = require("json-parse-even-better-errors");
					options = parseJson(options);
				} catch (e: any) {
					throw new Error(`Cannot parse string options: ${e.message}`);
				}
			} else {
				options = querystring.parse(options);
			}
		}

		if (options === null || options === undefined) {
			options = {};
		}

		return options;
	};

	let compilation: Compilation | undefined = compiler._lastCompilation;
	let step = 0;
	while (compilation) {
		NormalModule.getCompilationHooks(compilation).loader.call(loaderContext);
		compilation = compilation.compiler.parentCompilation;
		step++;
		if (step > 1000) {
			throw Error(
				"Too many nested child compiler, exceeded max limitation 1000"
			);
		}
	}

	/// Sync with `context`
	Object.defineProperty(loaderContext, "loaderIndex", {
		enumerable: true,
		get: () => context.loaderIndex,
		set: loaderIndex => (context.loaderIndex = loaderIndex)
	});
	Object.defineProperty(loaderContext, "cacheable", {
		enumerable: true,
		get: () => (cacheable: boolean) => {
			if (cacheable === false) {
				context.cacheable = cacheable;
			}
		}
	});
	Object.defineProperty(loaderContext, "data", {
		enumerable: true,
		get: () => loaderContext.loaders[loaderContext.loaderIndex].data,
		set: data => (loaderContext.loaders[loaderContext.loaderIndex].data = data)
	});

	switch (loaderState) {
		case JsLoaderState.Pitching: {
			while (loaderContext.loaderIndex < loaderContext.loaders.length) {
				const currentLoaderObject =
					loaderContext.loaders[loaderContext.loaderIndex];

				if (currentLoaderObject.shouldYield()) break;
				if (currentLoaderObject.pitchExecuted) {
					loaderContext.loaderIndex += 1;
					continue;
				}

				await loadLoaderAsync(currentLoaderObject);
				const fn = currentLoaderObject.pitch;
				currentLoaderObject.pitchExecuted = true;
				if (!fn) continue;

				const args =
					(await runSyncOrAsync(fn, loaderContext, [
						loaderContext.remainingRequest,
						loaderContext.previousRequest,
						currentLoaderObject.data
					])) || [];

				const hasArg = args.some(value => value !== undefined);

				if (hasArg) {
					const [content, sourceMap, additionalData] = args;
					context.content = isNil(content) ? null : toBuffer(content);
					context.sourceMap = serializeObject(sourceMap);
					context.additionalData = additionalData;
					break;
				}
			}

			break;
		}
		case JsLoaderState.Normal: {
			let content = context.content;
			let sourceMap = JsSourceMap.__from_binding(context.sourceMap);
			let additionalData = context.additionalData;

			while (loaderContext.loaderIndex >= 0) {
				const currentLoaderObject =
					loaderContext.loaders[loaderContext.loaderIndex];

				if (currentLoaderObject.shouldYield()) break;
				if (currentLoaderObject.normalExecuted) {
					loaderContext.loaderIndex--;
					continue;
				}

				await loadLoaderAsync(currentLoaderObject);
				const fn = currentLoaderObject.normal;
				currentLoaderObject.normalExecuted = true;
				if (!fn) continue;
				let args = [content, sourceMap, additionalData];
				convertArgs(args, !!currentLoaderObject.raw);
				[content, sourceMap, additionalData] =
					(await runSyncOrAsync(fn, loaderContext, args)) || [];
			}

			context.content = isNil(content) ? null : toBuffer(content);
			context.sourceMap = JsSourceMap.__to_binding(sourceMap);
			context.additionalData = additionalData;

			break;
		}
		default:
			throw new Error(`Unexpected loader runner state: ${loaderState}`);
	}

	// update loader state
	context.loaderItems = loaderContext.loaders.map(item =>
		LoaderObject.__to_binding(item)
	);

	return context;
}

function utf8BufferToString(buf: Buffer) {
	const str = buf.toString("utf-8");
	if (str.charCodeAt(0) === 0xfeff) {
		return str.slice(1);
	} else {
		return str;
	}
}

function convertArgs(args: any[], raw: boolean) {
	if (!raw && Buffer.isBuffer(args[0])) args[0] = utf8BufferToString(args[0]);
	else if (raw && typeof args[0] === "string")
		args[0] = Buffer.from(args[0], "utf-8");
}

const PATH_QUERY_FRAGMENT_REGEXP =
	/^((?:\u200b.|[^?#\u200b])*)(\?(?:\u200b.|[^#\u200b])*)?(#.*)?$/;

export function parsePathQueryFragment(str: string): {
	path: string;
	query: string;
	fragment: string;
} {
	const match = PATH_QUERY_FRAGMENT_REGEXP.exec(str);
	return {
		path: match?.[1].replace(/\u200b(.)/g, "$1") || "",
		query: match?.[2] ? match[2].replace(/\u200b(.)/g, "$1") : "",
		fragment: match?.[3] || ""
	};
}
