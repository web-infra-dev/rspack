import * as binding from "@rspack/binding";

export type ResolvedStatsOptions = binding.RawStatsOptions;

export interface StatsOptions {
	colors?: boolean;
	all?: boolean;
	warnings?: boolean;
	errors?: boolean;
}

export function resolveStatsOptions(
	options: StatsOptions = {}
): ResolvedStatsOptions {
	const colors = optionsOrFallback(options.colors, false);
	return {
		colors
	};
}

const optionsOrFallback = (...args) => {
	let optionValues = [];
	optionValues.push(...args);
	return optionValues.find((optionValue) => optionValue !== undefined);
};
