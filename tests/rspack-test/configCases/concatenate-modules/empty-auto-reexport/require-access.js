"use strict";
require("./empty.js?require-target");
globalThis.emptyAutoResolve = [
	require.resolve("./empty.js?require-target"),
	require.resolveWeak("./empty.js?require-target")
];
