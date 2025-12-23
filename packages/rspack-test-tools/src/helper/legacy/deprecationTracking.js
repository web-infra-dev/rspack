// @ts-nocheck
/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/

"use strict";

let interception = undefined;

const originalWarn = console.warn;

console.warn = (message, ...args) => {
	if (interception && typeof message === 'string' && message.includes('[Rspack Deprecation]')) {
		interception.set(message, {
			message,
			stack: new Error(message).stack
		});
	}
	return originalWarn.apply(console, [message, ...args]);
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
