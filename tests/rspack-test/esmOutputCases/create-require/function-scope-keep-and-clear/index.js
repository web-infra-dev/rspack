import { createRequire } from "node:module";

// The demand-driven keep/clear decision (module.parser.javascript.requireResolve: false)
// also applies to createRequire declared inside a function scope, not just module top level.

// Case C — the created require is used only for an analyzable `require(...)` invoke, which
// is bundled. The createRequire itself is then dead, so it is cleared to `undefined`.
function onlyInvoke() {
	const require = createRequire(import.meta.url);
	return require("./dep.js");
}

// Case B — the created require value is ACCESSED (a `.resolve` member here, not just an
// invoke), so the real createRequire must be kept (rendered via rspack's helper).
function accessValue() {
	const require = createRequire(import.meta.url);
	return require.resolve("path");
}

// Case A — the created require ESCAPES the function (returned out and re-exported), so it
// must be kept; the exported value has to be a real require.
function makeRequire() {
	const require = createRequire(import.meta.url);
	return require;
}

export const invoked = onlyInvoke();
export const resolved = accessValue();
export const escaped = makeRequire();

it("keeps a function-scoped createRequire only when accessed or escaping; clears invoke-only", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const source = fs.readFileSync(path.join(__dirname, "main.mjs"), "utf-8");

	// Built by concatenation so the marker isn't present as this file's own bundled source
	// (which would make the assertion pass without an actual clear having happened).
	const clearedMarker = "/* createRequire()" + " */ undefined";

	// Case C: the invoke-only createRequire is cleared. (Runtime alone can't show this — the
	// invoke is bundled whether or not the declaration is kept — so assert on the emitted
	// source.)
	expect(source).toContain(clearedMarker);

	// Case C runtime: the bundled `require("./dep.js")` still returns the dep, even though
	// the createRequire declaration was cleared (the cleared `undefined` is never read).
	expect(invoked).toBe(1);

	// Case B: `.resolve` runs at runtime — a cleared `undefined` would throw here, proving
	// the createRequire was kept. A builtin resolves to its own name at runtime.
	expect(resolved).toBe("path");

	// Case A: the value that escaped the function is the real created require, kept alive.
	expect(typeof escaped).toBe("function");
	expect(escaped.resolve("path")).toBe("path");
});
