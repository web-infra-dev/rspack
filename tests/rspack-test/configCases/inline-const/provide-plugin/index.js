const fs = __non_webpack_require__("fs");
const generated = /** @type {string} */ (fs.readFileSync(__filename, "utf-8"));

it("should provide inlined const exports", () => {
	// START:A
	expect(providedA).toBe(1);
	expect(providedDefault).toBe(2);
	// END:A
	const block = generated.match(/([\s\S]*?)\/\/ START:A[\s\S]*?\/\/ END:A/)[1];
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(block.includes(`/* provided dependency */ var providedA = (__rspack_context.r("./constants.js"), (/* inlined export .a */1));`)).toBe(true);
		expect(block.includes(`/* provided dependency */ var providedDefault = (__rspack_context.r("./constants.js"), (/* inlined export ["default"] */2));`)).toBe(true);
	} else {
		expect(block.includes(`/* provided dependency */ var providedA = (__webpack_require__("./constants.js"), (/* inlined export .a */1));`)).toBe(true);
		expect(block.includes(`/* provided dependency */ var providedDefault = (__webpack_require__("./constants.js"), (/* inlined export ["default"] */2));`)).toBe(true);
	}
	expect(globalThis.__rspackProvideConstSideEffect).toBe(1);
	delete globalThis.__rspackProvideConstSideEffect;
});
