import type { RawBuiltins } from "@rspack/binding";
import { loadConfig } from "browserslist";

export type Builtins = Omit<RawBuiltins, "browserslist">;

export type ResolvedBuiltins = RawBuiltins;

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
