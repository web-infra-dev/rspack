import { makeRequire } from "./shim.js";

// Deferred-shape declaration with a custom source: must keep `makeRequire`, NOT rewrite to
// Node's built-in createRequire.
const r = makeRequire(import.meta.url);
export const fromShimVar = r.__fromShim;
export const resolvedVar = r.resolve("path");

// Inline member access on a custom source: same requirement.
export const fromShimInline = makeRequire(import.meta.url).__fromShim;
export const resolvedInline = makeRequire(import.meta.url).resolve("path");

it("keeps a custom createRequire source instead of the built-in helper (requireResolve disabled)", () => {
	// The marker proves the runtime used the user's `makeRequire`, not Node's built-in
	// createRequire (which the helper would have substituted).
	expect(fromShimVar).toBe(true);
	expect(fromShimInline).toBe(true);
	// And the preserved created require still resolves at runtime.
	expect(resolvedVar).toBe("path");
	expect(resolvedInline).toBe("path");
});
