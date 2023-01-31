/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/test/helpers/deprecationTracking.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

const util = require("util");

let interception = undefined;

const originalDeprecate = util.deprecate;
util.deprecate = (fn, message, code) => {
	const original = originalDeprecate(fn, message, code);

	return function (...args) {
		if (interception) {
			interception.set(`${code}: ${message}`, {
				code,
				message,
				stack: new Error(message).stack
			});
			return fn.apply(this, args);
		} else {
			return original.apply(this, args);
		}
	};
};

exports.start = handler => {
	interception = new Map();

	return () => {
		const map = interception;
		interception = undefined;
		return Array.from(map || [])
			.sort(([a], [b]) => {
				if (a < b) return -1;
				if (a > b) return 1;
				return 0;
			})
			.map(([key, data]) => data);
	};
};
