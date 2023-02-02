export const memoize = <T>(fn: () => T): (() => T) => {
	let cache = false;
	// @ts-expect-error
	let result = undefined;
	return () => {
		if (cache) {
			// @ts-expect-error
			return result;
		} else {
			result = fn();
			cache = true;
			// Allow to clean up memory for fn
			// and all dependent resources
			// @ts-expect-error
			fn = undefined;
			return result;
		}
	};
};
