const fs = __non_webpack_require__("fs");
const generated = /** @type {string} */ (fs.readFileSync(__filename, "utf-8"));

it("should provide inlined const exports", () => {
	// START:A
	expect(providedA).toBe(1);
	expect(providedDefault).toBe(2);
	// END:A
	const block = generated.match(/([\s\S]*?)\/\/ START:A[\s\S]*?\/\/ END:A/)[1];
	const requireExpression = typeof __rspack_context !== "undefined"
		? "__rspack_context.r"
		: "__webpack_require__";
	expect(block.includes(`/* provided dependency */ var providedA = (${requireExpression}("./constants.js"), (/* inlined export .a */1));`)).toBe(true);
	expect(block.includes(`/* provided dependency */ var providedDefault = (${requireExpression}("./constants.js"), (/* inlined export ["default"] */2));`)).toBe(true);
	expect(globalThis.__rspackProvideConstSideEffect).toBe(1);
	delete globalThis.__rspackProvideConstSideEffect;
});
