import defer * as deferred from "./vanilla";
export { value } from "./vanilla";
export const local = 7;
export const readDeferred = () => deferred.value();

globalThis.LAZY_BARREL_DEFER_READY = import("./vanilla");
globalThis.LAZY_BARREL_DEFER_ACCEPTS = 0;

import.meta.webpackHot.accept("./vanilla", () => {
	globalThis.LAZY_BARREL_DEFER_ACCEPTS++;
});
import.meta.webpackHot.accept();
