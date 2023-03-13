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

export function isPromiseLike(value: unknown): value is Promise<any> {
	return (
		typeof value === "object" &&
		value !== null &&
		typeof (value as any).then === "function"
	);
}

export function concatErrorMsgAndStack(err: Error): string {
	return `${err.message}${err.stack ? `\n${err.stack}` : ""}`;
}

export function indent(str: string, prefix: string) {
	const rem = str.replace(/\n([^\n])/g, "\n" + prefix + "$1");
	return prefix + rem;
}

export function asArray<T>(item: T[]): T[]
export function asArray<T>(item: readonly T[]): readonly T[]
export function asArray<T>(item: T): T[]
export function asArray<T>(item: T | T[]): T[] {
	return Array.isArray(item) ? item : [item];
}
