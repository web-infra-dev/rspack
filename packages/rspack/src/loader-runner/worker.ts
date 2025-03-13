import querystring from "node:querystring";
import { promisify } from "node:util";
import { type MessagePort, receiveMessageOnPort } from "node:worker_threads";

import type { ResolveCallback, ResolveRequest } from "enhanced-resolve";
import type { LoaderObject } from ".";
import type { AssetInfo } from "../Compilation";
import type { Mode, Resolve, Target } from "../config";
import type {
	ImportModuleOptions,
	LoaderContextCallback,
	LoaderExperiments
} from "../config/adapterRuleUse";
import type { Logger } from "../logging/Logger";
import { createHash } from "../util/createHash";
import type Hash from "../util/hash";
import { absolutify, contextify } from "../util/identifier";
import { memoize } from "../util/memoize";
import loadLoader from "./loadLoader";
import {
	RequestSyncType,
	RequestType,
	type WorkerError,
	type WorkerMessage,
	type WorkerRequestMessage,
	type WorkerRequestSyncMessage,
	type WorkerResponseErrorMessage,
	type WorkerResponseMessage,
	isWorkerResponseErrorMessage,
	isWorkerResponseMessage,
	serializeError
} from "./service";
import { convertArgs, runSyncOrAsync } from "./utils";

interface WorkerLoaderObject {
	request: string;
	query: string;
	fragment: string;
	options?: string | object;
	raw?: boolean;
	normal?: Function;
	pitch?: Function;
	ident: string | null;
	type?: "module" | "commonjs";
}

interface WorkerLoaderContext<OptionsType = {}> {
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
	 * Note: This is not a Rspack public API, maybe removed in future.
	 * Store some data from loader, and consume it from parser, it may be removed in the future
	 *
	 * @internal
	 */
	__internal__parseMeta: Record<string, string>;

	__internal__workerInfo: {
		hashFunction: string;
	};
}

interface WorkerOptions {
	loaderObject: WorkerLoaderObject;
	loaderContext: WorkerLoaderContext;
	pitch: boolean;
	args: any[];

	workerData?: {
		workerPort: MessagePort;
		workerSyncPort: MessagePort;
	};
}

const loadLoaderAsync: (loaderObject: any) => Promise<void> =
	promisify(loadLoader);

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

async function loaderImpl(
	{ loaderObject, args, loaderContext, pitch }: WorkerOptions,
	sendRequest: SendRequestFunction
) {
	await loadLoaderAsync(loaderObject);

	if (!pitch) {
		convertArgs(args, !!loaderObject.raw);
	}
	//
	const resourcePath = loaderContext.resourcePath;
	const contextDirectory = resourcePath ? dirname(resourcePath) : null;

	loaderContext.dependency = loaderContext.addDependency =
		function addDependency(file) {
			sendRequest(RequestType.AddDependency, file);
		};
	loaderContext.addContextDependency = function addContextDependency(context) {
		sendRequest(RequestType.AddContextDependency, context);
	};
	loaderContext.addBuildDependency = function addBuildDependency(file) {
		sendRequest(RequestType.AddBuildDependency, file);
	};
	loaderContext.getDependencies = function getDependencies() {
		return [];
	};
	loaderContext.getContextDependencies = function getContextDependencies() {
		return [];
	};
	loaderContext.getMissingDependencies = function getMissingDependencies() {
		return [];
	};
	loaderContext.clearDependencies = function clearDependencies() {
		sendRequest(RequestType.ClearDependencies);
	};
	loaderContext.importModule = function () {
		throw new Error("importModule is not supported in worker");
	};
	loaderContext.resolve = function resolve(context, request, callback) {
		sendRequest(RequestType.Resolve, context, request).then(
			result => {
				callback(null, result);
			},
			err => {
				callback(err);
			}
		);
	};
	loaderContext.getResolve = function getResolve(options) {
		return (context, request, callback) => {
			sendRequest(RequestType.GetResolve, options, context, request).then(
				result => {
					callback(null, result);
				},
				err => {
					callback(err);
				}
			);
		};
	};
	loaderContext.getLogger = function getLogger(name) {
		return {
			error(...args) {
				sendRequest(RequestType.GetLogger, "error", name, args);
			},
			warn(...args) {
				sendRequest(RequestType.GetLogger, "warn", name, args);
			},
			info(...args) {
				sendRequest(RequestType.GetLogger, "info", name, args);
			},
			log(...args) {
				sendRequest(RequestType.GetLogger, "log", name, args);
			},
			debug(...args) {
				sendRequest(RequestType.GetLogger, "debug", name, args);
			},
			assert(assertion, ...args) {
				if (!assertion) {
					sendRequest(RequestType.GetLogger, "error", name, args);
				}
			},
			trace() {
				sendRequest(RequestType.GetLogger, "trace", name, ["Trace"]);
			},
			clear() {
				sendRequest(RequestType.GetLogger, "clear", name);
			},
			status(...args) {
				sendRequest(RequestType.GetLogger, "status", name, args);
			},
			group(...args) {
				sendRequest(RequestType.GetLogger, "group", name, args);
			},
			groupCollapsed(...args) {
				sendRequest(RequestType.GetLogger, "groupCollapsed", name, args);
			},
			groupEnd(...args) {
				sendRequest(RequestType.GetLogger, "groupEnd", name, args);
			},
			profile(label) {
				sendRequest(RequestType.GetLogger, "profile", name, [label]);
			},
			profileEnd(label) {
				sendRequest(RequestType.GetLogger, "profileEnd", name, [label]);
			},
			time(label) {
				sendRequest(RequestType.GetLogger, "time", name, [label]);
			},
			timeEnd(label) {
				sendRequest(RequestType.GetLogger, "timeEnd", name, [label]);
			},
			timeLog(label, ...args) {
				sendRequest(RequestType.GetLogger, "timeLog", name, [label, ...args]);
			},
			timeAggregate(label) {
				sendRequest(RequestType.GetLogger, "timeAggregate", name, [label]);
			},
			timeAggregateEnd(label) {
				sendRequest(RequestType.GetLogger, "timeAggregateEnd", name, [label]);
			}
		};
	} as WorkerLoaderContext["getLogger"];

	loaderContext.emitError = function emitError(err) {
		sendRequest(RequestType.EmitError, serializeError(err));
	};
	loaderContext.emitWarning = function emitWarning(warning) {
		sendRequest(RequestType.EmitWarning, serializeError(warning));
	};
	loaderContext.emitFile = function emitFile(
		name,
		content,
		sourceMap,
		assetInfo
	) {
		sendRequest(RequestType.EmitFile, name, content, sourceMap, assetInfo);
	};
	loaderContext.experiments = {
		emitDiagnostic(diagnostic) {
			sendRequest(RequestType.EmitDiagnostic, diagnostic);
		}
	};

	const getAbsolutify = memoize(() => absolutify.bindCache({}));
	const getAbsolutifyInContext = memoize(() =>
		absolutify.bindContextCache(contextDirectory!, {})
	);
	const getContextify = memoize(() => contextify.bindCache({}));
	const getContextifyInContext = memoize(() =>
		contextify.bindContextCache(contextDirectory!, {})
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
				type || loaderContext.__internal__workerInfo.hashFunction
			);
		}
	};

	Object.defineProperty(loaderContext, "_compiler", {
		enumerable: true,
		get: () => {
			throw new Error("`loaderContext._compiler` is not supported in worker");
		}
	});

	Object.defineProperty(loaderContext, "_compilation", {
		enumerable: true,
		get: () => {
			throw new Error(
				"`loaderContext._compilation` is not supported in worker"
			);
		}
	});

	Object.defineProperty(loaderContext, "_module", {
		enumerable: true,
		get: () => {
			throw new Error("`loaderContext._module` is not supported in worker");
		}
	});

	loaderContext.getOptions = function getOptions() {
		const loader = loaderObject;
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

	loaderContext.cacheable = function cacheable(cacheable: boolean) {
		if (cacheable === false) {
			sendRequest(RequestType.SetCacheable, false);
		}
	};

	const cachedLoaderContextData = memoize(() => {
		const data = sendRequest.sync(RequestSyncType.GetData) ?? {};
		return new Proxy(data, {
			set: (target, key, value) => {
				const newData = { ...data, [key]: value };
				sendRequest.sync(RequestSyncType.SetData, newData);
				return true;
			}
		});
	});
	Object.defineProperty(loaderContext, "data", {
		enumerable: true,
		get: () => cachedLoaderContextData
	});

	if (pitch) {
		return runSyncOrAsync(loaderObject.pitch!, loaderContext as any, [
			loaderContext.remainingRequest,
			loaderContext.previousRequest,
			cachedLoaderContextData
		]);
	}

	return runSyncOrAsync(loaderObject.normal!, loaderContext as any, args);
}

let nextId = 0;
const responseCallbacks: Record<
	number,
	(err: WorkerError | null, data: any) => void
> = {};

function handleIncomingResponses(workerMessage: WorkerMessage) {
	if (isWorkerResponseMessage(workerMessage)) {
		const { id, data } = workerMessage;
		const callback = responseCallbacks[id];
		if (callback) {
			delete responseCallbacks[id];
			callback(null, /* data */ data);
		} else {
			throw new Error(`No callback found for response with id ${id}`);
		}
	} else if (isWorkerResponseErrorMessage(workerMessage)) {
		const { id, error } = workerMessage;
		const callback = responseCallbacks[id];
		if (callback) {
			delete responseCallbacks[id];
			callback(error, undefined);
		} else {
			throw new Error(`No callback found for response with id ${id}`);
		}
	}
}

interface SendRequestFunction {
	(requestType: RequestType, ...args: any[]): Promise<any>;
	sync(requestType: RequestSyncType, ...args: any[]): any;
}

function createSendRequest(
	workerPort: MessagePort,
	workerSyncPort: MessagePort
): SendRequestFunction {
	const sendRequest = ((requestType, ...args) => {
		const id = nextId++;
		workerPort.postMessage({
			type: "request",
			id,
			requestType,
			data: args
		} satisfies WorkerRequestMessage);
		return new Promise((resolve, reject) => {
			responseCallbacks[id] = (err, data) => {
				if (err) {
					reject(err);
					return;
				}
				resolve(data);
			};
		});
	}) as SendRequestFunction;
	sendRequest.sync = createSendRequestSync(workerSyncPort);
	return sendRequest;
}

function createSendRequestSync(workerSyncPort: MessagePort) {
	return (requestType: RequestSyncType, ...args: any[]) => {
		const id = nextId++;

		// Create `sharedArrayBuffer` for each request.
		// This is used to synchronize between the main thread and worker thread.
		const sharedBuffer = new SharedArrayBuffer(8);
		const sharedBufferView = new Int32Array(sharedBuffer);

		workerSyncPort.postMessage({
			type: "request-sync",
			id,
			requestType,
			data: args,
			sharedBuffer
		} satisfies WorkerRequestSyncMessage);

		// Atomics.wait returns immediately with the value 'not-equal'
		// Otherwise, the thread is blocked until another thread calls Atomics.notify
		// with the same memory location or the timeout is reached.
		//
		// See: https://v8.dev/features/atomics
		const status = Atomics.wait(sharedBufferView, 0, 0);
		if (status !== "ok" && status !== "not-equal")
			throw new Error("Internal error: Atomics.wait() failed: " + status);

		const {
			message
		}: { message: WorkerResponseMessage | WorkerResponseErrorMessage } =
			receiveMessageOnPort(workerSyncPort)!;

		if (id !== message.id) {
			throw new Error(`Unexpected response id: ${message.id}, expected: ${id}`);
		}

		if (isWorkerResponseMessage(message)) {
			return message.data;
		}

		throw message.error;
	};
}

function worker(workerOptions: WorkerOptions) {
	const workerData = workerOptions.workerData!;
	delete workerOptions.workerData;

	workerData.workerPort.on("message", handleIncomingResponses);
	const sendRequest = createSendRequest(
		workerData.workerPort,
		workerData.workerSyncPort
	);

	loaderImpl(workerOptions, sendRequest)
		.then(async data => {
			workerData.workerPort.postMessage({ type: "done", data });
		})
		.catch(async err => {
			workerData.workerPort.postMessage({
				type: "done-error",
				error: serializeError(err)
			});
		});
}

export = worker;
