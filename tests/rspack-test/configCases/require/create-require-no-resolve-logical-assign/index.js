import { createRequire } from "module";

// A deferred created require used in a logical assignment must be kept: a truthy, non-null
// created require means `||=`/`??=` skip their RHS while `&&=` runs its RHS. Clearing the
// declaration to `undefined` would flip all of those, so the observed side effects below must
// match a real created require, not `undefined`.

let orRan = false;
let rOr = createRequire(import.meta.url);
rOr ||= (orRan = true); // truthy -> RHS must be skipped
export const orInvoke = rOr("./dep");

let nullishRan = false;
let rNullish = createRequire(import.meta.url);
rNullish ??= (nullishRan = true); // non-null -> RHS must be skipped
export const nullishResolved = rNullish.resolve("path");

let andRan = false;
let rAnd = createRequire(import.meta.url);
rAnd &&= (andRan = true); // truthy -> RHS must run

// Non-dominating reassignment: `rReset` is reassigned only inside a function that does NOT run
// during module evaluation, so the module-level `.resolve` below uses the real created
// require. The declaration must be kept (a cleared `undefined` would crash here at eval).
let rReset = createRequire(import.meta.url);
function reset() {
	rReset = 1;
}
export const resetResolved = rReset.resolve("path");
export { reset };

it("keeps a deferred created require used in a logical assignment", () => {
	// `||=` skipped its RHS because the created require is truthy (a cleared `undefined` would
	// have run it). The require() through the kept binding is still bundled.
	expect(orRan).toBe(false);
	expect(orInvoke).toBe(1);

	// `??=` skipped its RHS because the created require is non-null, and `.resolve` still runs.
	expect(nullishRan).toBe(false);
	expect(nullishResolved).toBe("path");

	// `&&=` ran its RHS because the created require is truthy (a cleared `undefined` would have
	// skipped it).
	expect(andRan).toBe(true);

	// Non-dominating reassignment: `reset` has not run, so `rReset` is still the real created
	// require and `.resolve` works (a cleared `undefined` would have thrown above).
	expect(resetResolved).toBe("path");
});
