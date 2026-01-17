const generated = /** @type {string} */ (__non_webpack_require__("fs").readFileSync(__filename, "utf-8"));

it("should use low-level syntax", () => {
	// START:A
	const a = [1, 2, 3];
	const b = [...a];
	// END:A
	const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
	expect(a).toEqual(b);
	expect(block.includes("...a")).toBe(false);
});
