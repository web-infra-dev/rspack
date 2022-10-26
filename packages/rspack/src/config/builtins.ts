import type { RawBuiltins } from "@rspack/binding";
import { loadConfig } from "browserslist";

export type Builtins = Omit<RawBuiltins, "browserslist"> & {
	polyfillBuiltins?: boolean; // polyfill node builtin api
};

export type ResolvedBuiltins = RawBuiltins & {
	polyfillBuiltins?: boolean;
};

function resolveDefine(define = {}) {
	const entries = Object.entries(define).map(([key, value]) => [
		key,
		JSON.stringify(value)
	]);
	return Object.fromEntries(entries);
}

export function resolveBuiltinsOptions(
	builtins: Builtins,
	contextPath: string
): ResolvedBuiltins {
	const browserslist = loadConfig({ path: contextPath }) || [];
	return {
		...builtins,
		browserslist
	};
}
