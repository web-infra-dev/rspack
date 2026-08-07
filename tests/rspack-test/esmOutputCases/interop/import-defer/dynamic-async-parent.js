import { dependency } from "./dynamic-async-dependency";

globalThis.__modern_module_defer_events__.push("dynamic-async-parent");

export const value = dependency;
