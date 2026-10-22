import { value } from './dep.js';

export const ready = new Promise((resolve) => {
	require(['exports'], function (exports) {
		globalThis.__amdEsmExports = exports;
		resolve(value);
	});
});
