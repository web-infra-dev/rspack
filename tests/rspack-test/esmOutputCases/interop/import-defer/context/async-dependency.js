await new Promise((resolve) => setTimeout(resolve, 10));
globalThis.__modern_module_defer_events__.push("context-async-dependency");

export const dependency = 7;
