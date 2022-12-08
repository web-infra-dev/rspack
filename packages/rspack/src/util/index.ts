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
