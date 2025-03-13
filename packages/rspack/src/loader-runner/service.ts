import path from "node:path";
import { MessageChannel } from "node:worker_threads";
import type { Tinypool } from "tinypool" with { "resolution-mode": "import" };

let pool: Promise<Tinypool> | undefined;
const ensureLoaderWorkerPool = async () => {
	if (pool) {
		return pool;
	}
	return (pool = import("tinypool").then(({ Tinypool }) => {
		const cpus = require("node:os").cpus().length;
		const availableThreads = Math.max(cpus - 1, 1);

		const pool = new Tinypool({
			filename: path.resolve(__dirname, "loaderWorker.js"),
			useAtomics: false,

			maxThreads: availableThreads,
			minThreads: availableThreads,
			concurrentTasksPerWorker: 1
		});

		return pool;
	}));
};

type RunOptions = Parameters<Tinypool["run"]>[1];

export interface WorkerResponseMessage {
	type: "response";
	id: number;

	data: any;
}

export interface WorkerResponseErrorMessage {
	type: "response-error";
	id: number;

	error: WorkerError;
}

interface WorkerDoneMessage {
	type: "done";
	// content, sourceMap, additionalData
	data: WorkerResult;
}

interface WorkerDoneErrorMessage {
	type: "done-error";
	error: WorkerError;
}

export interface WorkerRequestMessage {
	type: "request";
	id: number;

	requestType: RequestType;
	// arguments passed to the request
	data: any[];
}

export interface WorkerRequestSyncMessage {
	type: "request-sync";
	id: number;

	requestType: RequestSyncType;
	// arguments passed to the request
	data: any[];
	sharedBuffer: SharedArrayBuffer;
}
export type WorkerMessage =
	| WorkerResponseMessage
	| WorkerDoneMessage
	| WorkerRequestMessage
	| WorkerResponseErrorMessage
	| WorkerDoneErrorMessage
	| WorkerRequestSyncMessage;

export function isWorkerResponseMessage(
	message: WorkerMessage
): message is WorkerResponseMessage {
	return message.type === "response";
}

function isWorkerDoneMessage(
	message: WorkerMessage
): message is WorkerDoneMessage {
	return message.type === "done";
}

function isWorkerDoneErrorMessage(
	message: WorkerMessage
): message is WorkerDoneErrorMessage {
	return message.type === "done-error";
}

function isWorkerRequestMessage(
	message: WorkerMessage
): message is WorkerRequestMessage {
	return message.type === "request";
}

export function isWorkerResponseErrorMessage(
	message: WorkerMessage
): message is WorkerResponseErrorMessage {
	return message.type === "response-error";
}

export enum RequestType {
	AddDependency = "add-dependency",
	AddContextDependency = "add-context-dependency",
	AddMissingDependency = "add-missing-dependency",
	AddBuildDependency = "add-build-dependency",
	ClearDependencies = "clear-dependencies",
	Resolve = "resolve",
	GetResolve = "get-resolve",
	GetLogger = "get-logger",
	EmitError = "emit-error",
	EmitWarning = "emit-warning",
	EmitFile = "emit-file",
	EmitDiagnostic = "emit-diagnostic",
	SetCacheable = "set-cacheable"
}

export enum RequestSyncType {
	SetData = "set-data",
	GetData = "get-data"
}

export type HandleIncomingRequest = (
	requestType: RequestType,
	...args: any[]
) => Promise<any> | any;
export type HandleIncomingRequestSync = (
	requestType: RequestSyncType,
	...args: any[]
) => any;

// content, sourceMap, additionalData
type WorkerResult = any[];

export type WorkerError = Error;

export function serializeError(error: unknown): WorkerError {
	if (
		error instanceof Error ||
		(error && typeof error === "object" && "message" in error)
	) {
		// Consider object with messaage property as an error
		return {
			...error,
			name: (error as Error).name,
			stack: (error as Error).stack,
			message: (error as Error).message
		};
	}

	if (typeof error === "string") {
		return {
			name: "Error",
			message: error
		};
	}

	throw new Error(
		"Failed to serialize error, only string, Error instances and objects with a message property are supported"
	);
}

export const run = async (
	task: any,
	options: RunOptions & {
		handleIncomingRequest: HandleIncomingRequest;
		handleIncomingRequestSync: HandleIncomingRequestSync;
	}
) =>
	ensureLoaderWorkerPool().then(async pool => {
		const { port1: mainPort, port2: workerPort } = new MessageChannel();
		// Create message channel for processing sync API requests from worker
		// threads.
		const { port1: mainSyncPort, port2: workerSyncPort } = new MessageChannel();
		return new Promise<WorkerResult>((resolve, reject) => {
			const handleError = (error: any) => {
				mainPort.close();
				mainSyncPort.close();
				reject(error);
			};
			const pendingRequests: Promise<any>[] = [];
			mainPort.on("message", (message: WorkerMessage) => {
				if (isWorkerDoneMessage(message)) {
					Promise.allSettled(pendingRequests).then(p => {
						mainPort.close();
						mainSyncPort.close();
						resolve(message.data);
					});
				} else if (isWorkerDoneErrorMessage(message)) {
					Promise.allSettled(pendingRequests).then(() => {
						mainPort.close();
						mainSyncPort.close();
						reject(message.error);
					});
				} else if (isWorkerRequestMessage(message)) {
					pendingRequests.push(
						Promise.resolve()
							.then(() =>
								options.handleIncomingRequest(
									message.requestType,
									...message.data
								)
							)
							.then(result => {
								mainPort.postMessage({
									type: "response",
									id: message.id,
									data: result
								} satisfies WorkerResponseMessage);
							})
							.catch(error => {
								mainPort.postMessage({
									type: "response-error",
									id: message.id,
									error: serializeError(error)
								} satisfies WorkerResponseErrorMessage);
							})
					);
				}
			});
			mainPort.on("messageerror", handleError);
			mainSyncPort.on("message", (message: WorkerRequestSyncMessage) => {
				const sharedBuffer = message.sharedBuffer;
				const sharedBufferView = new Int32Array(sharedBuffer);

				let result;
				try {
					result = options.handleIncomingRequestSync(
						message.requestType,
						...message.data
					);

					mainSyncPort.postMessage({
						type: "response",
						id: message.id,
						data: result
					} satisfies WorkerResponseMessage);
				} catch (e: unknown) {
					mainSyncPort.postMessage({
						type: "response-error",
						id: message.id,
						error: serializeError(e)
					} satisfies WorkerResponseErrorMessage);
				}

				// If `Atomics.wait` on the worker side is called after this
				// `Atomics.add` call, `Atomics.wait` will return immediately
				// without putting the worker to sleep.
				Atomics.add(sharedBufferView, 0, 1);

				// Otherwise, if `Atomics.wait` is called before this `Atomics.add` call,
				// We uses `Atomics.notify` to wake up the worker instead.
				Atomics.notify(sharedBufferView, 0, Number.POSITIVE_INFINITY);
			});
			mainSyncPort.on("messageerror", handleError);
			pool
				.run(
					{
						...task,
						// Internal worker data. Tinypool does not support passing `transferList` to
						// `new Worker(..)`
						workerData: {
							workerPort,
							workerSyncPort
						}
					},
					{
						...options,
						transferList: [
							...(options?.transferList || []),
							workerPort,
							workerSyncPort
						]
					}
				)
				.catch(handleError);
		});
	});
