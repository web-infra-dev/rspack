import { RawSplitChunksOptions, RawCacheGroupOptions } from "@rspack/binding";

/**
 * Options object for splitting chunks into smaller chunks.
 */
export interface OptimizationSplitChunksOptions {
	/**
	 * Sets the name delimiter for created chunks.
	 */
	// automaticNameDelimiter?: string;
	/**
	 * Assign modules to a cache group (modules from different cache groups are tried to keep in separate chunks, default categories: 'default', 'defaultVendors').
	 */
	cacheGroups?: {
		/**
		 * Configuration for a cache group.
		 */
		[k: string]: // | false
		// | RegExp
		// | string
		// | Function
		OptimizationSplitChunksCacheGroup;
	};
	/**
	 * Select chunks for determining shared modules (defaults to "async", "initial" and "all" requires adding these chunks to the HTML).
	 */
	chunks?: "initial" | "async" | "all";
	// | ((chunk: import("../lib/Chunk")) => boolean);
	/**
	 * Sets the size types which are used when a number is used for sizes.
	 */
	// defaultSizeTypes?: string[];
	/**
	 * Size threshold at which splitting is enforced and other restrictions (minRemainingSize, maxAsyncRequests, maxInitialRequests) are ignored.
	 */
	enforceSizeThreshold?: OptimizationSplitChunksSizes;
	/**
	 * Options for modules not selected by any other cache group.
	 */
	// fallbackCacheGroup?: {
	//     /**
	//      * Sets the name delimiter for created chunks.
	//      */
	//     automaticNameDelimiter?: string;
	//     /**
	//      * Select chunks for determining shared modules (defaults to "async", "initial" and "all" requires adding these chunks to the HTML).
	//      */
	//     chunks?:
	//     | ("initial" | "async" | "all")
	//     // | ((chunk: import("../lib/Chunk")) => boolean);
	//     /**
	//      * Maximal size hint for the on-demand chunks.
	//      */
	//     maxAsyncSize?: OptimizationSplitChunksSizes;
	//     /**
	//      * Maximal size hint for the initial chunks.
	//      */
	//     maxInitialSize?: OptimizationSplitChunksSizes;
	//     /**
	//      * Maximal size hint for the created chunks.
	//      */
	//     maxSize?: OptimizationSplitChunksSizes;
	//     /**
	//      * Minimal size for the created chunk.
	//      */
	//     minSize?: OptimizationSplitChunksSizes;
	//     /**
	//      * Minimum size reduction due to the created chunk.
	//      */
	//     minSizeReduction?: OptimizationSplitChunksSizes;
	// };
	/**
	 * Sets the template for the filename for created chunks.
	 */
	// filename?:
	// | string
	// | ((
	//     pathData: import("../lib/Compilation").PathData,
	//     assetInfo?: import("../lib/Compilation").AssetInfo
	// ) => string);
	/**
	 * Prevents exposing path info when creating names for parts splitted by maxSize.
	 */
	// hidePathInfo?: boolean;
	/**
	 * Maximum number of requests which are accepted for on-demand loading.
	 */
	maxAsyncRequests?: number;
	/**
	 * Maximal size hint for the on-demand chunks.
	 */
	maxAsyncSize?: OptimizationSplitChunksSizes;
	/**
	 * Maximum number of initial chunks which are accepted for an entry point.
	 */
	maxInitialRequests?: number;
	/**
	 * Maximal size hint for the initial chunks.
	 */
	maxInitialSize?: OptimizationSplitChunksSizes;
	/**
	 * Maximal size hint for the created chunks.
	 */
	maxSize?: OptimizationSplitChunksSizes;
	/**
	 * Minimum number of times a module has to be duplicated until it's considered for splitting.
	 */
	minChunks?: number;
	/**
	 * Minimal size for the chunks the stay after moving the modules to a new chunk.
	 */
	minRemainingSize?: OptimizationSplitChunksSizes;
	/**
	 * Minimal size for the created chunks.
	 */
	minSize?: OptimizationSplitChunksSizes;
	/**
	 * Minimum size reduction due to the created chunk.
	 */
	minSizeReduction?: OptimizationSplitChunksSizes;
	/**
	 * Give chunks created a name (chunks with equal name are merged).
	 */
	name?: false | string | Function;
	/**
	 * Compare used exports when checking common modules. Modules will only be put in the same chunk when exports are equal.
	 */
	// usedExports?: boolean;
}

/**
 * Options object for describing behavior of a cache group selecting modules that should be cached together.
 */
export interface OptimizationSplitChunksCacheGroup {
	/**
	 * Sets the name delimiter for created chunks.
	 */
	automaticNameDelimiter?: string;
	chunks?: "initial" | "async" | "all";
	enforce?: boolean;
	enforceSizeThreshold?: OptimizationSplitChunksSizes;
	filename?: string;
	idHint?: string;
	layer?: RegExp | string | Function;
	maxAsyncRequests?: number;

	maxAsyncSize?: OptimizationSplitChunksSizes;
	maxInitialRequests?: number;
	maxInitialSize?: OptimizationSplitChunksSizes;
	maxSize?: OptimizationSplitChunksSizes;
	minChunks?: number;
	minRemainingSize?: OptimizationSplitChunksSizes;
	minSize?: OptimizationSplitChunksSizes;
	minSizeReduction?: OptimizationSplitChunksSizes;
	name?: string;
	priority?: number;
	reuseExistingChunk?: boolean;
	test?: RegExp;
}

/**
 * Size description for limits.
 */
export type OptimizationSplitChunksSizes = number;
// | {
//     /**
//      * Size of the part of the chunk with the type of the key.
//      */
//     [k: string]: number;
// };

export function resolveSplitChunksOptions(
	op?: OptimizationSplitChunksOptions
): undefined | RawSplitChunksOptions {
	if (!op || op == null) {
		return undefined;
	} else {
		op.cacheGroups ??= {};
		return {
			chunks: op.chunks,
			maxAsyncRequests: op.maxAsyncRequests,
			maxInitialRequests: op.maxInitialRequests,
			minChunks: op.minChunks,
			minSize: op.minSize,
			enforceSizeThreshold: op.enforceSizeThreshold,
			minRemainingSize: op.minRemainingSize,
			cacheGroups: op.cacheGroups
				? Object.fromEntries(
						Object.entries(op.cacheGroups).map(([key, group]) => {
							let normalizedGroup: RawCacheGroupOptions = {
								test: group.test ? group.test.source : undefined,
								name: group.name,
								priority: group.priority,
								minChunks: group.minChunks,
								chunks: group.chunks
							};
							return [key, normalizedGroup];
						})
				  )
				: undefined
		};
	}
}
