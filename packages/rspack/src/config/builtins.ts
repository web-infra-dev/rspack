import type { RawBuiltins } from "@rspack/binding";
import { loadConfig } from "browserslist";

export type Builtins = Omit<RawBuiltins, "browserslist">;

export type ResolvedBuiltins = RawBuiltins;

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
		browserslist,
		define: resolveDefine(builtins.define)
	};
}
