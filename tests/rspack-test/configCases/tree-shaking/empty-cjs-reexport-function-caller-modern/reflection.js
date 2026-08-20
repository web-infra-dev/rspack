globalThis.modernFunctionCallerTargetLoaded = true;

function getFactoryExports() {
	return getFactoryExports.caller.arguments[1];
}
