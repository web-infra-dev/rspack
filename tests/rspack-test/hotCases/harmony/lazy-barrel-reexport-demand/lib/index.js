globalThis.LAZY_BARREL_READY = import("./vanilla");
globalThis.LAZY_BARREL_ACCEPTS = 0;
export { value } from "./vanilla";
export const local = 7;

module.hot.accept("./vanilla", () => {
	globalThis.LAZY_BARREL_ACCEPTS++;
});

module.hot.accept();
---
globalThis.LAZY_BARREL_READY = import("./vanilla");
globalThis.LAZY_BARREL_ACCEPTS = 0;
export { value } from "./vanilla";
export const local = 7;

module.hot.accept("./vanilla", () => {
	globalThis.LAZY_BARREL_ACCEPTS++;
});

module.hot.accept();
---
globalThis.LAZY_BARREL_READY = import("./vanilla");
globalThis.LAZY_BARREL_ACCEPTS = 0;
export { value } from "./vanilla";
export const local = 7;

module.hot.accept("./vanilla", () => {
	globalThis.LAZY_BARREL_ACCEPTS++;
});

module.hot.accept();
---
globalThis.LAZY_BARREL_READY = import("./vanilla");
globalThis.LAZY_BARREL_ACCEPTS = 0;
export { value } from "./vanilla";
export const local = 7;

module.hot.accept("./vanilla", () => {
	globalThis.LAZY_BARREL_ACCEPTS++;
});

module.hot.accept();
export const revision = 1;
