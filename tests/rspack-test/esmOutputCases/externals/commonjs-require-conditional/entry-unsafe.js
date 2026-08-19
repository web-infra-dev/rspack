globalThis.__USE_EXTERNAL__ = true;
globalThis.__RUN_DYNAMIC_CONTEXT__ = false;
globalThis.__USE_CONTEXT_EXTERNAL__ = false;
globalThis.__LOCAL_MODULE__ = "local.cjs";

const { selected, contextual } = require("./unsafe.cjs");

process.__mixedExternalValues.push(selected);

if (contextual.value !== 7) {
	throw new Error("dynamic CommonJS context fallback should keep working");
}

export const value = selected.value;
