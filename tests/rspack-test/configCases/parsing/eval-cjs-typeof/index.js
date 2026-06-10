const fs = require("fs");

it("should compile", async () => {
	if (typeof exports !== "object" || typeof module !== "object") {
		throw new Error("wrong")
	}
	const file = await fs.promises.readFile(__filename, 'utf-8');
	if (file.includes(["var ", "__rspack_context"].join(""))) {
		expect(file).not.toContain(['if (', 'ty', 'pe', 'of exports !== "object"', ' || ', 'ty', 'pe', 'of module !== "object"', ')'].join(""))
	} else {
		expect(file).not.toContain(["ty", "pe", "of"].join(""))
	}
});
