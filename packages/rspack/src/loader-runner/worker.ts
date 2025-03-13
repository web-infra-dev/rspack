import type { MessagePort } from "node:worker_threads";
// @ts-nocheck
import { ResolverFactory } from "../ResolverFactory";

interface WorkerLoaderObject {
	request: string;
	query: string;
	fragment: string;
	options?: string;
	raw?: boolean;
	normal?: Function;
	pitch?: Function;
	ident: string | null;
	type?: "module" | "commonjs";
}

interface WorkerOptions {
	loaderObject: WorkerLoaderObject;

	args: any[];

	tx: MessagePort;
}

import { promisify } from "node:util";
import type {
	LoaderContext,
	LoaderContextCallback
} from "../config/adapterRuleUse";
import { memoize } from "../util/memoize";
import loadLoader from "./loadLoader";
import { convertArgs } from "./utils";

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

const loadLoaderAsync: (loaderObject: any) => Promise<void> =
	promisify(loadLoader);

async function loaderImpl({
	loaderObject,
	args,
	loaderContext
}: WorkerOptions) {
	// console.log("before loadloader", loaderObject, args)
	await loadLoaderAsync(loaderObject);
	convertArgs(args, !!loaderObject.raw);

	//
	// console.log(loaderObject)
	//
	loaderContext.getOptions = function getOptions() {
		return {};
	};

	loaderContext.addDependency = () => {};

	// TODO: pnp
	const getResolver = memoize(() => {
		return new ResolverFactory(false).get("normal");
	});

	// TODO:
	const getResolveContext = () => {
		return {
			fileDependencies: {
				// @ts-expect-error: Mocking insert-only `Set<T>`
				add: d => {
					// loaderContext.addDependency(d);
				}
			},
			contextDependencies: {
				// @ts-expect-error: Mocking insert-only `Set<T>`
				add: d => {
					// loaderContext.addContextDependency(d);
				}
			},
			missingDependencies: {
				// @ts-expect-error: Mocking insert-only `Set<T>`
				add: d => {
					// loaderContext.addMissingDependency(d);
				}
			}
		};
	};

	loaderContext.getLogger = function getLogger() {
		return {
			error() {},
			warn() {},
			info() {}
		};
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
	const newArgs = await runSyncOrAsync(
		loaderObject.normal!,
		loaderContext,
		args
	);

	return { newArgs };
}

function worker(workerOptions: WorkerOptions) {
	loaderImpl(workerOptions)
		.then(({ newArgs }) => {
			workerOptions.tx.postMessage({ newArgs });
		})
		.catch(e => {
			throw e;
		});
}
export = worker;
