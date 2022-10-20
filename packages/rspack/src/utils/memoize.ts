export const memoize = <T>(fn: () => T): (() => T) => {
	let cache = false;
	let result = undefined;
	return () => {
		if (cache) {
			return result;
		} else {
			result = fn();
			cache = true;
			// Allow to clean up memory for fn
			// and all dependent resources
			fn = undefined;
			return result;
		}
	};
};
