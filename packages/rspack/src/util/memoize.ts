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

export function memoizeValue<T>(fn: () => T): T {
	const getValue: () => any = memoize(fn);
	return new Proxy({} as any, {
		get(_, property) {
			let res = getValue()[property];
			if (typeof res === "function") {
				res = res.bind(getValue());
			}
			return res;
		},
		set(_, property, newValue) {
			getValue()[property] = newValue;
			return true;
		},
		deleteProperty(_, property) {
			const value = getValue();
			return delete value[property];
		},
		has: (_, property) => {
			return property in getValue();
		},
		ownKeys: _ => {
			return Object.keys(getValue());
		},
		getOwnPropertyDescriptor(_, property) {
			return Object.getOwnPropertyDescriptor(getValue(), property);
		}
	});
}
