import { createRequire } from "module";

// A top-level export shares its name with an UNRELATED function-scoped created require. The
// function-scoped one is used only for a bundled invoke, so it must be cleared. The keep
// decision is resolved through the scope-aware createRequire tag, so the exported top-level
// name must not keep the function-scoped declaration — otherwise the function body would
// retain a literal URL expression that is a syntax error in this CommonJS bundle. The
// forbidden substrings are checked from test.config.js so they are not part of this source.
export const req = 1;

export function load() {
	const req = createRequire(import.meta.url);
	return req("./dep.js");
}

it("clears a function-scoped createRequire that only shadows an exported name", () => {
	// The bundle loaded at all (no syntax error), and the bundled require() still works.
	expect(req).toBe(1);
	expect(load()).toBe(1);
});
