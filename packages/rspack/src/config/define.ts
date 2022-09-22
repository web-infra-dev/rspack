export type Define = Record<string, string>;

export type ResolvedDefine = Record<string, string>;

export function resolveDefine(define = {}) {
	const entries = Object.entries(define).map(([key, value]) => [
		key,
		JSON.stringify(value)
	]);
	return Object.fromEntries(entries);
}
