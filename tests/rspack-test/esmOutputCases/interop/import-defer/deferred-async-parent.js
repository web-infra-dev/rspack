import { dependency } from "./async-dependency";

globalThis.__modern_module_defer_events__.push("async-parent");

export const value = dependency;
