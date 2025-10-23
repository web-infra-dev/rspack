import path from "node:path";
// biome-ignore syntax/correctness/noTypeOnlyImportAttributes: Biome does not support this
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
			filename: path.resolve(__dirname, "worker.js"),
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
	data: WorkerArgs;
}

interface WorkerDoneErrorMessage {
	type: "done-error";
	error: WorkerError;
}

export interface WorkerRequestMessage {
	type: "request";
	id: number;

	requestType: RequestType;
	data: WorkerArgs;
}

export interface WorkerRequestSyncMessage {
	type: "request-sync";
	id: number;

	requestType: RequestSyncType;
	data: WorkerArgs;
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
	AddDependency = "AddDependency",
	AddContextDependency = "AddContextDependency",
	AddMissingDependency = "AddMissingDependency",
	AddBuildDependency = "AddBuildDependency",
	GetDependencies = "GetDependencies",
	GetContextDependencies = "GetContextDependencies",
	GetMissingDependencies = "GetMissingDependencies",
	ClearDependencies = "ClearDependencies",
	Resolve = "Resolve",
	GetResolve = "GetResolve",
	GetLogger = "GetLogger",
	EmitError = "EmitError",
	EmitWarning = "EmitWarning",
	EmitFile = "EmitFile",
	EmitDiagnostic = "EmitDiagnostic",
	SetCacheable = "SetCacheable",
	ImportModule = "ImportModule",
	UpdateLoaderObjects = "UpdateLoaderObjects",
	CompilationGetPath = "CompilationGetPath",
	CompilationGetPathWithInfo = "CompilationGetPathWithInfo",
	CompilationGetAssetPath = "CompilationGetAssetPath",
	CompilationGetAssetPathWithInfo = "CompilationGetAssetPathWithInfo"
}

export enum RequestSyncType {
	WaitForPendingRequest = "WaitForPendingRequest"
}

export type HandleIncomingRequest = (
	requestType: RequestType,
	...args: any[]
) => any;

// content, sourceMap, additionalData
type WorkerArgs = any[];

export type WorkerError = Error;

export function serializeError(error: unknown): WorkerError {
	if (
		error instanceof Error ||
		(error && typeof error === "object" && "message" in error)
	) {
		// Consider object with message property as an error
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
// check which props are not cloneable
function checkCloneableProps(obj: any, loaderName: string) {
	const errors = [];

	for (const key of Object.keys(obj)) {
		try {
			structuredClone(obj[key]);
		} catch (e: any) {
			errors.push({ key, type: typeof obj[key], reason: e.message });
		}
	}

	if (errors.length > 0) {
		const errorMsg = errors
			.map(
				err =>
					`option "${err.key}" (type: ${err.type}) is not cloneable: ${err.reason}`
			)
			.join("\n");

		throw new Error(
			`The options for ${loaderName} are not cloneable, which is not supported by parallelLoader. Consider disabling parallel for this loader or removing the non-cloneable properties from the options:\n${errorMsg}`
		);
	}
}

export const run = async (
	loaderName: string,
	task: any,
	options: RunOptions & {
		handleIncomingRequest: HandleIncomingRequest;
	}
) =>
	ensureLoaderWorkerPool().then(async pool => {
		const { MessageChannel } = await import("node:worker_threads");
		const { port1: mainPort, port2: workerPort } = new MessageChannel();
		// Create message channel for processing sync API requests from worker
		// threads.
		const { port1: mainSyncPort, port2: workerSyncPort } = new MessageChannel();
		return new Promise<WorkerArgs>((resolve, reject) => {
			const handleError = (error: any) => {
				mainPort.close();
				mainSyncPort.close();
				reject(error);
			};
			const pendingRequests: Map<number, Promise<any>> = new Map();
			mainPort.on("message", (message: WorkerMessage) => {
				if (isWorkerDoneMessage(message)) {
					Promise.allSettled(pendingRequests.values()).then(() => {
						mainPort.close();
						mainSyncPort.close();
						resolve(message.data);
					});
				} else if (isWorkerDoneErrorMessage(message)) {
					Promise.allSettled(pendingRequests.values()).then(() => {
						mainPort.close();
						mainSyncPort.close();
						reject(message.error);
					});
				} else if (isWorkerRequestMessage(message)) {
					pendingRequests.set(
						message.id,
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
								return result;
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
			// eslint-disable-next-line @typescript-eslint/no-misused-promises
			mainSyncPort.on("message", async (message: WorkerRequestSyncMessage) => {
				const { sharedBuffer } = message;
				const sharedBufferView = new Int32Array(sharedBuffer);

				let result: any;
				try {
					switch (message.requestType) {
						case RequestSyncType.WaitForPendingRequest: {
							const pendingRequestId = message.data[0];
							const isArray = Array.isArray(pendingRequestId);

							const ids = isArray ? pendingRequestId : [pendingRequestId];
							// Pending requests now are not returning errors.
							// To handle errors, you should not call `wait()` on send request
							// result;
							result = await Promise.all(
								ids.map(id => pendingRequests.get(id))
							);

							if (!isArray) {
								result = result[0];
							}
							break;
						}
						default:
							throw new Error(`Unknown request type: ${message.requestType}`);
					}

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
			checkCloneableProps(task, loaderName);
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
