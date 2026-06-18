import depDefault, { req, exportedAlias, copy, assigned } from "./dep";

it("keeps exported created requires (named, aliased, default, declarator/assignment copy) for cross-module `.resolve`", () => {
	// If any of dep.js's createRequire had been cleared to `undefined`, accessing `.resolve`
	// would throw `Cannot read properties of undefined`.
	for (const r of [req, exportedAlias, depDefault, copy, assigned]) {
		expect(typeof r).toBe("function");
		// `.resolve` runs at runtime: a builtin resolves to its own name.
		expect(r.resolve("path")).toBe("path");
	}
});
