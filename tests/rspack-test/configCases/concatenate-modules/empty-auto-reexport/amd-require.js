"use strict";
globalThis.emptyAutoAmdRequire = new Promise(resolve => {
	require(["./empty.js?require-target"], () => resolve(true));
});
