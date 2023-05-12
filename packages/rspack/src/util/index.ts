import { JsAssetInfo, JsStatsError } from "@rspack/binding";
import { AssetInfo } from "../compilation";

export function mapValues(
	record: Record<string, string>,
	fn: (key: string) => string
) {
	return Object.fromEntries(
		Object.entries(record).map(([key, value]) => [key, fn(value)])
	);
}

export function isNil(value: unknown): value is null | undefined {
	return value === null || value === undefined;
}

export const toBuffer = (bufLike: string | Buffer): Buffer => {
	if (Buffer.isBuffer(bufLike)) {
		return bufLike;
	} else if (typeof bufLike === "string") {
		return Buffer.from(bufLike);
	}

	throw new Error("Buffer or string expected");
};

export const toObject = (input: string | Buffer | object): object => {
	let s: string;
	if (Buffer.isBuffer(input)) {
		s = input.toString("utf8");
	} else if (input && typeof input === "object") {
		return input;
	} else if (typeof input === "string") {
		s = input;
	} else {
		throw new Error("Buffer or string or object expected");
	}

	return JSON.parse(s);
};

export function isPromiseLike(value: unknown): value is Promise<any> {
	return (
		typeof value === "object" &&
		value !== null &&
		typeof (value as any).then === "function"
	);
}

export function isJsStatsError(err: any): err is JsStatsError {
	return !(err instanceof Error) && err.formatted;
}

export function concatErrorMsgAndStack(err: Error | JsStatsError): string {
	// deduplicate the error if message is already shown in the stack
	//@ts-ignore
	const stackStartPrefix = err.name ? `${err.name}: ` : "Error: ";
	return isJsStatsError(err)
		? err.formatted
		: err.stack
		? err.stack.startsWith(`${stackStartPrefix}${err.message}`)
			? `${err.stack}`
			: `${err.message}\n${err.stack}`
		: `${err.message}`;
}

export function indent(str: string, prefix: string) {
	const rem = str.replace(/\n([^\n])/g, "\n" + prefix + "$1");
	return prefix + rem;
}

export function asArray<T>(item: T[]): T[];
export function asArray<T>(item: readonly T[]): readonly T[];
export function asArray<T>(item: T): T[];
export function asArray<T>(item: T | T[]): T[] {
	return Array.isArray(item) ? item : [item];
}

export function toJsAssetInfo(info?: AssetInfo): JsAssetInfo {
	return {
		immutable: false,
		minimized: false,
		development: false,
		hotModuleReplacement: false,
		related: {},
		contentHash: [],
		...info
	};
}
