import * as rspackExports from "./exports";
import { rspack as rspackFn } from "./rspack";

// add exports on rspack() function
type Rspack = typeof rspackFn &
	typeof rspackExports & { rspack: Rspack; webpack: Rspack };
// NOTE: Don't eagerly `Object.assign(rspackFn, rspackExports)` here.
// `import * as rspackExports` is a module namespace object whose properties are getters.
// Accessing them during module initialization can trigger ESM TDZ/circular init issues.
// Instead, define lazy getters on the function to preserve the webpack-compatible surface.
const fn = rspackFn as Rspack;
for (const key of Object.keys(rspackExports)) {
	// Some bundlers may already attach a subset of exports onto the function object.
	// Avoid redefining non-configurable properties.
	if (Object.prototype.hasOwnProperty.call(fn, key)) continue;
	Object.defineProperty(fn, key, {
		configurable: true,
		enumerable: true,
		get() {
			return (rspackExports as any)[key];
		}
	});
}
fn.rspack = fn;
fn.webpack = fn;
const rspack: Rspack = fn;
// Expose the full rspack API for internal consumers (e.g. `Compiler`) without creating
// a static import cycle between `./exports` and `./Compiler`.
if (typeof globalThis !== "undefined") {
	(globalThis as any).__RSPACK_WEBPACK_API__ = rspack;
}

export * from "./exports";
export default rspack;
export { rspack };
