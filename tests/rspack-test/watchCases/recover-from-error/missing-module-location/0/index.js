it("should keep the missing-module location stable after an incremental rebuild", function () {
	// Build ./uses-dep.js as a separate chunk but do not execute it, so the
	// missing-module error is reported at build time regardless of runtime.
	const load = () => import("./uses-dep.js");
	expect(typeof load).toBe("function");
});
