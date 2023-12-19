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

// Lazily init a function, and cache it afterwards.
export const memoizeFn = <const T extends readonly unknown[], const P>(
	fn: () => (...args: T) => P
) => {
	let cache: ((...args: T) => P) | null = null;
	return (...args: T) => {
		if (!cache) {
			cache = fn();
		}
		return cache(...args);
	};
};
