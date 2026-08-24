"use strict";

require(["module"], currentModule => {
	currentModule.exports.value = 42;
	globalThis.emptyAutoReexportAmdRequireExecuted = true;
});
