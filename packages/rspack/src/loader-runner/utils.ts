import path from "node:path";
import { promisify } from "node:util";
import type {
	LoaderContext,
	LoaderContextCallback
} from "../config/adapterRuleUse";
import type { Compiler } from "../exports";
import type { LoaderObject } from ".";
import loadLoaderRaw from "./loadLoader";

const decoder = new TextDecoder();

function utf8BufferToString(buf: Uint8Array) {
	// The provided ArrayBufferView value must not be shared.
	// ArrayBuffer or Uint8Array objects from other contexts (i.e. iframes) do not pass
	// the `instanceof` check but they should be treated as of that type.
	const isShared =
		buf.buffer instanceof SharedArrayBuffer ||
		buf.buffer.constructor?.name === "SharedArrayBuffer";

	const str = decoder.decode(isShared ? Buffer.from(buf) : buf);
	if (str.charCodeAt(0) === 0xfeff) {
		return str.slice(1);
	}
	return str;
}

export function convertArgs(args: any[], raw: boolean) {
	if (!raw && args[0] instanceof Uint8Array)
		args[0] = utf8BufferToString(args[0]);
	else if (raw && typeof args[0] === "string")
		args[0] = Buffer.from(args[0], "utf-8");

	// Ensure `Buffer` is used instead of `Uint8Array`
	if (raw && args[0] instanceof Uint8Array && !Buffer.isBuffer(args[0])) {
		args[0] = Buffer.from(args[0].buffer);
	}
}

export const loadLoader: (
	loaderObject: LoaderObject,
	compiler: Compiler
) => Promise<void> = promisify(loadLoaderRaw);

export const runSyncOrAsync = promisify(function runSyncOrAsync(
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
// from rsdoctor https://github.com/web-infra-dev/rsdoctor/blob/02e6cc484320083712ad922c4fff5ed41e960e25/packages/core/src/build-utils/build/utils/loader.ts#L56
// extract human readable loader name from loader path
export function extractLoaderName(loaderPath: string, cwd = ""): string {
	let res = loaderPath.replace(cwd, "");

	if (!path.isAbsolute(res)) return res;

	const nms = "/node_modules/";
	const idx = res.lastIndexOf(nms);

	if (idx !== -1) {
		// babel-loader/lib/index.js
		res = res.slice(idx + nms.length);

		const ln = "loader";
		const lnIdx = res.lastIndexOf(ln);
		if (lnIdx > -1) {
			// babel-loader
			res = res.slice(0, lnIdx + ln.length);
		}
	}

	return res;
}
