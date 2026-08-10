// Accessing the module object keeps this case behind a wrapper boundary.
module.forceWrapper = true;
globalThis.__strictCjsExecutions =
	(globalThis.__strictCjsExecutions || 0) + 1;
throw new Error("wrapped CommonJS failure");
