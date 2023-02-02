import * as binding from "@rspack/binding";
import { normalizeStatsPreset, optionsOrFallback } from "../stats";

export type ResolvedStatsOptions = binding.RawStatsOptions;

export interface StatsOptionsObj {
	all?: boolean;
	preset?: "normal" | "none" | "verbose" | "errors-only" | "errors-warnings";
	assets?: boolean;
	chunks?: boolean;
	modules?: boolean;
	entrypoints?: boolean;
	warnings?: boolean;
	warningsCount?: boolean;
	errors?: boolean;
	errorsCount?: boolean;
	colors?: boolean;
	hash?: boolean;
	reasons?: boolean;
	children?: StatsOptionsObj[];
	publicPath?: boolean;
}

export type StatsOptions = StatsOptionsObj | boolean | string;

/**
 * resolve `StatsOptions` to `binding.RawStatsOptions`.
 */
export function resolveStatsOptions(
	opts: StatsOptions = {}
): ResolvedStatsOptions {
	const options = normalizeStatsPreset(opts);
	const colors = optionsOrFallback(options.colors, false);
	return {
		...options,
		// @ts-expect-error
		colors
	};
}
