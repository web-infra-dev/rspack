function getFactoryExports() {
	return getFactoryExports.caller.arguments[1];
}

getFactoryExports().functionCallerOwnValue = "own exports";
