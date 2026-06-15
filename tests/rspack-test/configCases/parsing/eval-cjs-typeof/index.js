const fs = require("fs");

it("should compile", async () => {
	if (typeof exports !== "object" || typeof module !== "object") {
		throw new Error("wrong")
	}
	const file = await fs.promises.readFile(__filename, 'utf-8');
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(file).not.toContain(['if (', 'ty', 'pe', 'of exports !== "object"', ' || ', 'ty', 'pe', 'of module !== "object"', ')'].join(""))
	} else {
		expect(file).not.toContain(["ty", "pe", "of"].join(""))
	}
});
