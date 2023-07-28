/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/util
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

export const last = <T>(set: Iterable<T>): T | undefined => {
	let last;
	for (const item of set) last = item;
	return last;
};

export const someInIterable = <T>(
	iterable: Iterable<T>,
	filter: (arg: T) => boolean
): boolean => {
	for (const item of iterable) {
		if (filter(item)) return true;
	}
	return false;
};

export const countIterable = <T>(iterable: Iterable<T>): number => {
	let i = 0;
	// eslint-disable-next-line no-unused-vars
	for (const _ of iterable) i++;
	return i;
};
