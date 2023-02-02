import type { RawCacheOptions } from "@rspack/binding";

export type Cache =
	| boolean
	| {
			type: "memory";
			maxGenerations?: number;
	  }
	| {
			type: "filesystem";
			maxAge?: number;
			profile?: boolean;
			buildDependencies?: string[];
			cacheDirectory?: string;
			cacheLocation?: string;
			name?: string;
			version?: string;
	  };

export type ResolvedCache = RawCacheOptions;

export function resolveCacheOptions(cache: Cache): ResolvedCache {
	const result = {
		type: "",
		maxGenerations: 0,
		maxAge: 0,
		profile: false,
		buildDependencies: [],
		cacheDirectory: "",
		cacheLocation: "",
		name: "",
		version: ""
	};

	if (cache === false) {
		return result;
	}

	if (cache === true) {
		return { ...result, type: "memory" };
	}

	if (cache.type === "memory") {
		// @ts-expect-error
		cache.maxGenerations = isFinite(cache.maxGenerations)
			? cache.maxGenerations
			: 0;
	}

	return {
		...result,
		...cache
	};
}
