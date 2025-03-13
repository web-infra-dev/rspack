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

import assert from "node:assert";
import { promisify } from "node:util";
import {
	type JsLoaderContext,
	type JsLoaderItem,
	JsLoaderState,
	JsRspackSeverity,
	formatDiagnostic
} from "@rspack/binding";
import {
	OriginalSource,
	RawSource,
	type Source,
	SourceMapSource
} from "webpack-sources";

import type { ContextAPI, PropagationAPI, TraceAPI } from "@rspack/tracing";
import type { Compilation } from "../Compilation";
import type { Compiler } from "../Compiler";
import { NormalModule } from "../NormalModule";
import { NonErrorEmittedError, type RspackError } from "../RspackError";
import {
	BUILTIN_LOADER_PREFIX,
	type Diagnostic,
	type LoaderContext,
	type LoaderContextCallback,
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
	const obj: any = {
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
			const splittedRequest = parseResourceWithoutFragment(value.request);
			obj.path = splittedRequest.path;
			obj.query = splittedRequest.query;
			obj.fragment = "";
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

			// CHANGE: `rspack_core` returns empty string for `undefined` type.
			// Comply to webpack test case: tests/webpack-test/cases/loaders/cjs-loader-type/index.js
			obj.type = value.type === "" ? undefined : value.type;
			if (obj.options === null) obj.query = "";
			else if (obj.options === undefined) obj.query = "";
			else if (typeof obj.options === "string") obj.query = `?${obj.options}`;
			else if (obj.ident) obj.query = `??${obj.ident}`;
			else if (typeof obj.options === "object" && obj.options.ident)
				obj.query = `??${obj.options.ident}`;
			else obj.query = `?${JSON.stringify(obj.options)}`;
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
	callback: (err: Error | null | undefined, args: any[]) => void
) {
	let isSync = true;
	let isDone = false;
	let isError = false; // internal error
	let reportedError = false;
	context.async = function async() {
		if (isDone) {
			if (reportedError) return undefined as any; // ignore
			throw new Error("async(): The callback was already called.");
		}
		isSync = false;
		return innerCallback;
	};
	const innerCallback: LoaderContextCallback = (err, ...args) => {
		if (isDone) {
			if (reportedError) return; // ignore
			throw new Error("callback(): The callback was already called.");
		}
		isDone = true;
		isSync = false;
		try {
			callback(err, args);
		} catch (e) {
			isError = true;
			throw e;
		}
	};
	context.callback = innerCallback;

	try {
		const result = (function LOADER_EXECUTION() {
			return fn.apply(context, args);
		})();
		if (isSync) {
			isDone = true;
			if (result === undefined) {
				callback(null, []);
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
		// use string for napi getter
		const err = e as Error;
		if ("hideStack" in err && err.hideStack) {
			err.hideStack = "true";
		}
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
		callback(e as Error, []);
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
		loaderContext.loaders?.length &&
		index < loaderContext.loaders.length &&
		index >= 0 &&
		loaderContext.loaders[index]
	) {
		return loaderContext.loaders[index];
	}
	return null;
}

// FIXME: a temporary fix, we may need to change @rspack/tracing to commonjs really fix it
let cachedTracing:
	| {
			trace: TraceAPI;
			propagation: PropagationAPI;
			context: ContextAPI;
	  }
	| null
	| undefined;

async function getCachedTracing() {
	// disable tracing in non-profile mode
	if (!process.env.RSPACK_PROFILE) {
		cachedTracing = null;
		return cachedTracing;
	}
	if (cachedTracing) {
		return cachedTracing;
	}
	if (cachedTracing === undefined) {
		try {
			const tracing = await import("@rspack/tracing");
			cachedTracing = {
				trace: tracing.trace,
				propagation: tracing.propagation,
				context: tracing.context
			};
			return cachedTracing;
		} catch (e) {
			cachedTracing = null;
			return cachedTracing;
		}
	} else {
		cachedTracing = null;
		return cachedTracing;
	}
}

async function tryTrace(context: JsLoaderContext) {
	const cachedTracing = await getCachedTracing();
	if (cachedTracing) {
		const { trace, propagation, context: tracingContext } = cachedTracing;
		const tracer = trace.getTracer("rspack-loader-runner");
		const activeContext = propagation.extract(
			tracingContext.active(),
			context.__internal__tracingCarrier
		);
		return { trace, tracer, activeContext };
	}
	return null;
}

export async function runLoaders(
	compiler: Compiler,
	context: JsLoaderContext
): Promise<JsLoaderContext> {
	const { tracer, activeContext } = (await tryTrace(context)) ?? {};

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
		userOptions,
		callback
	) {
		const options = userOptions ? userOptions : {};
		const context = loaderContext;
		function finalCallback(
			onError: (err: Error) => void,
			onDone: (res: any) => void
		) {
			return function (err?: Error, res?: any) {
				if (err) {
					onError(err);
				} else {
					for (const dep of res.buildDependencies) {
						context.addBuildDependency(dep);
					}
					for (const dep of res.contextDependencies) {
						context.addContextDependency(dep);
					}
					for (const dep of res.missingDependencies) {
						context.addMissingDependency(dep);
					}
					for (const dep of res.fileDependencies) {
						context.addDependency(dep);
					}
					if (res.cacheable === false) {
						context.cacheable(false);
					}

					if (res.error) {
						onError(
							compiler.__internal__getModuleExecutionResult(res.id) ??
								new Error(err)
						);
					} else {
						onDone(compiler.__internal__getModuleExecutionResult(res.id));
					}
				}
			};
		}
		if (!callback) {
			return new Promise((resolve, reject) => {
				compiler
					._lastCompilation!.__internal_getInner()
					.importModule(
						request,
						options.layer,
						options.publicPath,
						options.baseUri,
						context._module.identifier(),
						loaderContext.context,
						finalCallback(reject, resolve)
					);
			});
		}
		return compiler._lastCompilation!.__internal_getInner().importModule(
			request,
			options.layer,
			options.publicPath,
			options.baseUri,
			context._module.identifier(),
			loaderContext.context,
			finalCallback(
				err => callback(err),
				res => callback(undefined, res)
			)
		);
	} as LoaderContext["importModule"];
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
		: (context._module.useSourceMap ?? false);
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

	const getResolver = memoize(() => {
		return compiler._lastCompilation!.resolverFactory.get("normal");
	});

	loaderContext.resolve = function resolve(context, request, callback) {
		getResolver().resolve({}, context, request, getResolveContext(), callback);
	};

	loaderContext.getResolve = function getResolve(options) {
		const resolver = getResolver();
		const child = options ? resolver.withOptions(options) : resolver;
		return (context, request, callback) => {
			if (callback) {
				child.resolve({}, context, request, getResolveContext(), callback);
				return;
			}
			// TODO: (type) our native resolver return value is "string | false" but webpack type is "string"
			return new Promise<string | false | undefined>((resolve, reject) => {
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
		};
	};
	loaderContext.getLogger = function getLogger(name) {
		return compiler._lastCompilation!.getLogger(
			[name, resource].filter(Boolean).join("|")
		);
	};
	loaderContext.rootContext = compiler.context;
	loaderContext.emitError = function emitError(err) {
		let error = err;
		if (!(error instanceof Error)) {
			error = new NonErrorEmittedError(error);
		}
		error.name = "ModuleError";
		error.message = `${error.message} (from: ${stringifyLoaderObject(
			loaderContext.loaders[loaderContext.loaderIndex]
		)})`;
		error = concatErrorMsgAndStack(error);
		(error as RspackError).moduleIdentifier =
			loaderContext._module.identifier();
		compiler._lastCompilation!.__internal__pushRspackDiagnostic({
			error,
			severity: JsRspackSeverity.Error
		});
	};
	loaderContext.emitWarning = function emitWarning(warn) {
		let warning = warn;
		if (!(warning instanceof Error)) {
			warning = new NonErrorEmittedError(warning);
		}
		warning.name = "ModuleWarning";
		warning.message = `${warning.message} (from: ${stringifyLoaderObject(
			loaderContext.loaders[loaderContext.loaderIndex]
		)})`;
		warning = concatErrorMsgAndStack(warning);
		(warning as RspackError).moduleIdentifier =
			loaderContext._module.identifier();
		compiler._lastCompilation!.__internal__pushRspackDiagnostic({
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
		let source: Source | undefined = undefined;
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

			if (loaderContext.sourceMap) {
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
		loaderContext._module.emitFile(name, source!, assetInfo!);
	};
	loaderContext.fs = compiler.inputFileSystem;
	loaderContext.experiments = {
		emitDiagnostic: (diagnostic: Diagnostic) => {
			const d = Object.assign({}, diagnostic, {
				message:
					diagnostic.severity === "warning"
						? `ModuleWarning: ${diagnostic.message}`
						: `ModuleError: ${diagnostic.message}`,
				moduleIdentifier: context._module.identifier()
			});
			compiler._lastCompilation!.__internal__pushDiagnostic(
				formatDiagnostic(d)
			);
		}
	};

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
				type || compiler._lastCompilation!.outputOptions.hashFunction!
			);
		}
	};

	loaderContext._compiler = compiler;
	loaderContext._compilation = compiler._lastCompilation!;
	loaderContext._module = context._module;

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
		NormalModule.getCompilationHooks(compilation).loader.call(
			loaderContext,
			loaderContext._module
		);
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
	Object.defineProperty(loaderContext, "__internal__parseMeta", {
		enumerable: true,
		get: () => context.__internal__parseMeta
	});

	try {
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

					const span = tracer?.startSpan(
						"LoaderRunner:pitch",
						{
							attributes: {
								"loader.identifier": getCurrentLoader(loaderContext)?.request
							}
						},
						activeContext
					);
					const args =
						(await runSyncOrAsync(fn, loaderContext, [
							loaderContext.remainingRequest,
							loaderContext.previousRequest,
							currentLoaderObject.data
						])) || [];
					span?.end();

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
					const args = [content, sourceMap, additionalData];
					convertArgs(args, !!currentLoaderObject.raw);

					const span = tracer?.startSpan(
						"LoaderRunner:normal",
						{
							attributes: {
								"loader.identifier": getCurrentLoader(loaderContext)?.request
							}
						},
						activeContext
					);
					[content, sourceMap, additionalData] =
						(await runSyncOrAsync(fn, loaderContext, args)) || [];
					span?.end();
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
	} catch (e) {
		const error = e as Error & { hideStack?: boolean | "true" };
		context.__internal__error =
			typeof e === "string"
				? {
						name: "ModuleBuildError",
						message: e
					}
				: {
						name: "ModuleBuildError",
						message: error.message,
						stack: typeof error.stack === "string" ? error.stack : undefined,
						hideStack:
							"hideStack" in error
								? error.hideStack === true || error.hideStack === "true"
								: undefined
					};
	}
	return context;
}

function utf8BufferToString(buf: Buffer) {
	const str = buf.toString("utf-8");
	if (str.charCodeAt(0) === 0xfeff) {
		return str.slice(1);
	}
	return str;
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
