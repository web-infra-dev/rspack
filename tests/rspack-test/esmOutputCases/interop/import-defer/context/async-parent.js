import { dependency } from "./async-dependency";

globalThis.__modern_module_defer_events__.push("context-async-parent");

export const value = dependency;
