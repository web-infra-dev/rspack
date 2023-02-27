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

export function sortObjectByKey<T extends Record<string, any>, K extends keyof T, V extends T[K]>(unordered: T): Record<string, V> {
	return Object.keys(unordered).sort().reduce<Record<string, V>>((obj, key) => {
		obj[key] = unordered[key]
		return obj
	}, {})
}
