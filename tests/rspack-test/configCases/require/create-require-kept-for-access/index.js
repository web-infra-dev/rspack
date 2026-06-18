import { createRequire } from "module";

const r = createRequire(import.meta.url);

// Value-position use: passing the created require to a function must keep the real
// createRequire (a webpack context-require could not resolve a builtin).
function id(x) {
	return x;
}
export const passed = id(r);

it("keeps a real created require when accessed by member or value (requireResolve disabled)", () => {
	// The value passed out is the real created require, not a context require.
	expect(typeof passed).toBe("function");
	expect(passed.resolve("path")).toBe("path");

	// A non-resolve member access reaches the real createRequire (Node require.resolve.paths),
	// instead of being rewritten to an unsupported `undefined`.
	expect(typeof r.resolve.paths).toBe("function");

	// The same chained member used INLINE on createRequire (not via a variable) must also
	// keep the real require, instead of becoming `undefined(...)`. It returns the same result
	// as the variable form (a cleared `undefined.resolve.paths(...)` would throw).
	expect(createRequire(import.meta.url).resolve.paths("path")).toEqual(r.resolve.paths("path"));

	// The inline member as a VALUE (not immediately called) must be kept too — otherwise it
	// would be rewritten to `undefined` and `paths(...)` would throw.
	const paths = createRequire(import.meta.url).resolve.paths;
	expect(typeof paths).toBe("function");
	expect(paths("path")).toEqual(r.resolve.paths("path"));

	// A MULTI-ARGUMENT inline createRequire member chain must be preserved as the original
	// call (the helper can't be used — it would drop the extra argument's side effect). It
	// must not collapse to `undefined(...)`, the side effect must run, and the chain stays.
	let multiArgRan = false;
	const multiPaths = createRequire(import.meta.url, (multiArgRan = true)).resolve.paths("path");
	expect(multiArgRan).toBe(true);
	expect(multiPaths).toEqual(r.resolve.paths("path"));

	// Invoking it is still bundled.
	expect(r("./dep")).toBe(1);

	// And `.resolve` itself is still preserved at runtime.
	expect(r.resolve("path")).toBe("path");
});
